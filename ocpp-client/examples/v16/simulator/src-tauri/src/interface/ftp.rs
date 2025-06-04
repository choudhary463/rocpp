use std::time::Duration;

use anyhow::anyhow;
use async_ftp::FtpStream;
use regex::Regex;
use tokio::io::AsyncReadExt;

pub struct FtpService {
    url: String,
}

impl FtpService {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub async fn upload(
        &self,
        file_name: String,
    ) -> anyhow::Result<()> {
        tokio::time::sleep(Duration::from_secs(2)).await;
        match self.get_ftp_stream().await {
            Ok((mut ftp_stream, path)) => {
                if let Some(path) = path {
                    ftp_stream.cwd(&path).await?;
                }
                let mut res = format!("LOGS START -----------\n");
                // add custom logic
                res += "LOGS END -----------";
                let mut cursor = std::io::Cursor::new(res.as_bytes());
                ftp_stream
                    .put(&file_name, &mut cursor)
                    .await
                    .map_err(|e| e.into())
            }
            Err(e) => Err(e),
        }
    }
    pub async fn download(&self) -> anyhow::Result<Vec<u8>> {
        let (mut ftp_stream, path) = self.get_ftp_stream().await?;
        let path = path.ok_or(anyhow!("path not found"))?;
        let mut reader = ftp_stream.simple_retr(&path).await?;
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).await?;
        Ok(contents)
    }
    async fn get_ftp_stream(&self) -> anyhow::Result<(FtpStream, Option<String>)> {
        let re = Regex::new(
            r"^ftp://(?P<user>[^:]+):(?P<password>[^@]+)@(?P<host>[^:/]+)(?::(?P<port>\d+))?(?P<path>/.*)?$",
        )?;

        let (user, password, host, port, path) = re
            .captures(&self.url)
            .map(|cap| {
                let user = cap["user"].to_string();
                let password = cap["password"].to_string();
                let host = cap["host"].to_string();
                let port = cap
                    .name("port")
                    .map(|p| p.as_str().parse::<u16>())
                    .transpose();
                let path = cap.name("path").map(|p| p.as_str().to_string());

                (user, password, host, port, path)
            })
            .ok_or(anyhow!("capture error"))?;

        let port = port?;

        let ftp_url = host + ":" + port.unwrap_or(21).to_string().as_str();

        let mut ftp_stream = FtpStream::connect(ftp_url).await?;
        ftp_stream.login(&user, &password).await?;
        Ok((ftp_stream, path))
    }
}