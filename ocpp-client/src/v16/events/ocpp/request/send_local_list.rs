use alloc::{string::String, vec::Vec};
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::send_local_list::{SendLocalListRequest, SendLocalListResponse},
        types::{AuthorizationData, UpdateStatus, UpdateType},
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    state_machine::{auth::LocalListChange},
    cp::core::ChargePointCore
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    fn send_local_list_ocpp_helper(&mut self, req: SendLocalListRequest) -> UpdateStatus {
        if !self.configs.local_auth_list_enabled.value {
            return UpdateStatus::NotSupported;
        }

        if req
            .local_authorization_list
            .as_ref()
            .is_some_and(|x| x.len() > self.configs.send_local_list_max_length.value)
        {
            return UpdateStatus::Failed;
        }

        let mut changes = Vec::new();
        let auth_list = req.local_authorization_list.unwrap_or_default();
        match req.update_type {
            UpdateType::Differential => {
                if req.list_version <= self.local_list_version {
                    return UpdateStatus::VersionMismatch;
                }

                let mut net_delta = 0;

                if auth_list.windows(2).any(|w| w[0].id_tag == w[1].id_tag) {
                    return UpdateStatus::Failed;
                }

                for AuthorizationData {
                    id_tag,
                    id_tag_info,
                } in auth_list
                {
                    match id_tag_info {
                        Some(info) => {
                            if !self.local_list_entries.contains_key(&id_tag) {
                                net_delta += 1;
                            }
                            changes.push(LocalListChange::Upsert {
                                id_tag: id_tag.clone(),
                                info,
                            });
                        }
                        None => {
                            if self.local_list_entries.contains_key(&id_tag) {
                                net_delta -= 1;
                                changes.push(LocalListChange::Delete {
                                    id_tag: id_tag.clone(),
                                });
                            }
                        }
                    }
                }
                if (self.local_list_entries.len() as isize + net_delta)
                    > self.configs.local_auth_list_max_length.value as isize
                {
                    return UpdateStatus::Failed;
                }
            }
            UpdateType::Full => {
                for AuthorizationData {
                    id_tag,
                    id_tag_info,
                } in auth_list
                {
                    if let Some(info) = id_tag_info {
                        changes.push(LocalListChange::Upsert {
                            id_tag: id_tag.clone(),
                            info: info.clone(),
                        });
                    } else {
                        return UpdateStatus::Failed;
                    }
                }

                if changes.len() >= self.configs.local_auth_list_max_length.value {
                    return UpdateStatus::Failed;
                }

                for old_tag in self.local_list_entries.keys() {
                    if changes.iter().all(|f| f.get_id_tag() != old_tag) {
                        changes.push(LocalListChange::Delete {
                            id_tag: old_tag.clone(),
                        });
                    }
                }
            }
        }

        for change in changes.clone() {
            match change {
                LocalListChange::Upsert { id_tag, info } => {
                    self.remove_from_cache(&id_tag);
                    self.local_list_entries.insert(id_tag, info);
                }
                LocalListChange::Delete { id_tag } => {
                    self.local_list_entries.remove(&id_tag);
                }
            }
        }
        let list_version = if self.local_list_entries.is_empty() {
            0
        } else {
            req.list_version
        };
        self.db.db_update_local_list(list_version, changes);
        self.local_list_version = list_version;
        UpdateStatus::Accepted
    }

    pub(crate) fn send_local_list_ocpp(&mut self, unique_id: String, req: SendLocalListRequest) {
        let status = self.send_local_list_ocpp_helper(req);
        let payload = SendLocalListResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
