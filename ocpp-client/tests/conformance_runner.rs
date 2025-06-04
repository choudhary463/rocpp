use std::future::Future;

use tokio::task::LocalSet;

mod conformance;
mod harness;
mod state;

async fn run_in_local<F>(fut: F)
where
    F: Future<Output = ()> + 'static,
{
    let local = LocalSet::new();
    let handle = local.spawn_local(async move {
        fut.await;
    });
    local.run_until(handle).await.unwrap();
}

#[tokio::test]
async fn tc_001_cs() {
    run_in_local(conformance::tc_001_cs::run()).await;
}

#[tokio::test]
async fn tc_002_cs() {
    run_in_local(conformance::tc_002_cs::run()).await;
}

#[tokio::test]
async fn tc_003_cs() {
    run_in_local(conformance::tc_003_cs::run()).await;
}

#[tokio::test]
async fn tc_004_1_cs() {
    run_in_local(conformance::tc_004_1_cs::run()).await;
}

#[tokio::test]
async fn tc_004_2_cs() {
    run_in_local(conformance::tc_004_2_cs::run()).await;
}

#[tokio::test]
async fn tc_068_cs() {
    run_in_local(conformance::tc_068_cs::run()).await;
}

#[tokio::test]
async fn tc_069_cs() {
    run_in_local(conformance::tc_069_cs::run()).await;
}

#[tokio::test]
async fn tc_005_2_cs() {
    run_in_local(conformance::tc_005_2_cs::run()).await;
}

#[tokio::test]
async fn tc_005_3_cs() {
    run_in_local(conformance::tc_005_3_cs::run()).await;
}

#[tokio::test]
async fn tc_007_cs() {
    run_in_local(conformance::tc_007_cs::run()).await;
}

#[tokio::test]
async fn tc_061_cs() {
    run_in_local(conformance::tc_061_cs::run()).await;
}

#[tokio::test]
async fn tc_010_cs() {
    run_in_local(conformance::tc_010_cs::run()).await;
}

#[tokio::test]
async fn tc_011_1_cs() {
    run_in_local(conformance::tc_011_1_cs::run()).await;
}

#[tokio::test]
async fn tc_011_2_cs() {
    run_in_local(conformance::tc_011_2_cs::run()).await;
}

#[tokio::test]
async fn tc_012_cs() {
    run_in_local(conformance::tc_012_cs::run()).await;
}

#[tokio::test]
async fn tc_013_cs() {
    run_in_local(conformance::tc_013_cs::run()).await;
}

#[tokio::test]
async fn tc_014_cs() {
    run_in_local(conformance::tc_014_cs::run()).await;
}

#[tokio::test]
async fn tc_015_cs() {
    run_in_local(conformance::tc_015_cs::run()).await;
}

#[tokio::test]
async fn tc_016_cs() {
    run_in_local(conformance::tc_016_cs::run()).await;
}

#[tokio::test]
async fn tc_017_2_cs() {
    run_in_local(conformance::tc_017_2_cs::run()).await;
}

#[tokio::test]
async fn tc_018_2_cs() {
    run_in_local(conformance::tc_018_2_cs::run()).await;
}

#[tokio::test]
async fn tc_019_cs() {
    run_in_local(conformance::tc_019_cs::run()).await;
}

#[tokio::test]
async fn tc_021_cs() {
    run_in_local(conformance::tc_021_cs::run()).await;
}

#[tokio::test]
async fn tc_070_cs() {
    run_in_local(conformance::tc_070_cs::run()).await;
}

#[tokio::test]
async fn tc_071_cs() {
    run_in_local(conformance::tc_071_cs::run()).await;
}

#[tokio::test]
async fn tc_023_cs() {
    run_in_local(conformance::tc_023_cs::run()).await;
}

#[tokio::test]
async fn tc_026_cs() {
    run_in_local(conformance::tc_026_cs::run()).await;
}

#[tokio::test]
async fn tc_027_cs() {
    run_in_local(conformance::tc_027_cs::run()).await;
}

#[tokio::test]
async fn tc_028_cs() {
    run_in_local(conformance::tc_028_cs::run()).await;
}

#[tokio::test]
async fn tc_031_cs() {
    run_in_local(conformance::tc_031_cs::run()).await;
}

#[tokio::test]
async fn tc_032_2_cs() {
    run_in_local(conformance::tc_032_2_cs::run()).await;
}

#[tokio::test]
async fn tc_034_cs() {
    run_in_local(conformance::tc_034_cs::run()).await;
}

#[tokio::test]
async fn tc_036_cs() {
    run_in_local(conformance::tc_036_cs::run()).await;
}

#[tokio::test]
async fn tc_037_1_cs() {
    run_in_local(conformance::tc_037_1_cs::run()).await;
}

#[tokio::test]
async fn tc_037_2_cs() {
    run_in_local(conformance::tc_037_2_cs::run()).await;
}

#[tokio::test]
async fn tc_037_3_cs() {
    run_in_local(conformance::tc_037_3_cs::run()).await;
}

#[tokio::test]
async fn tc_038_cs() {
    run_in_local(conformance::tc_038_cs::run()).await;
}

#[tokio::test]
async fn tc_039_cs() {
    run_in_local(conformance::tc_039_cs::run()).await;
}

#[tokio::test]
async fn tc_040_1_cs() {
    run_in_local(conformance::tc_040_1_cs::run()).await;
}

#[tokio::test]
async fn tc_040_2_cs() {
    run_in_local(conformance::tc_040_2_cs::run()).await;
}

#[tokio::test]
async fn tc_041_cs() {
    run_in_local(conformance::tc_041_cs::run()).await;
}

#[tokio::test]
async fn tc_042_1_cs() {
    run_in_local(conformance::tc_042_1_cs::run()).await;
}

#[tokio::test]
async fn tc_042_2_cs() {
    run_in_local(conformance::tc_042_2_cs::run()).await;
}

#[tokio::test]
async fn tc_043_cs() {
    run_in_local(conformance::tc_043_cs::run()).await;
}

#[tokio::test]
async fn tc_043_1_cs() {
    run_in_local(conformance::tc_043_1_cs::run()).await;
}

#[tokio::test]
async fn tc_043_2_cs() {
    run_in_local(conformance::tc_043_2_cs::run()).await;
}

#[tokio::test]
async fn tc_043_3_cs() {
    run_in_local(conformance::tc_043_3_cs::run()).await;
}

#[tokio::test]
async fn tc_008_cs() {
    run_in_local(conformance::tc_008_cs::run()).await;
}

#[tokio::test]
async fn tc_044_1_cs() {
    run_in_local(conformance::tc_044_1_cs::run()).await;
}

#[tokio::test]
async fn tc_044_2_cs() {
    run_in_local(conformance::tc_044_2_cs::run()).await;
}

#[tokio::test]
async fn tc_044_3_cs() {
    run_in_local(conformance::tc_044_3_cs::run()).await;
}

#[tokio::test]
async fn tc_045_1_cs() {
    run_in_local(conformance::tc_045_1_cs::run()).await;
}

#[tokio::test]
async fn tc_045_2_cs() {
    run_in_local(conformance::tc_045_2_cs::run()).await;
}

#[tokio::test]
async fn tc_046_1_cs() {
    run_in_local(conformance::tc_046_1_cs::run()).await;
}

#[tokio::test]
async fn tc_046_2_cs() {
    run_in_local(conformance::tc_046_2_cs::run()).await;
}

#[tokio::test]
async fn tc_047_cs() {
    run_in_local(conformance::tc_047_cs::run()).await;
}

#[tokio::test]
async fn tc_048_1_cs() {
    run_in_local(conformance::tc_048_1_cs::run()).await;
}

#[tokio::test]
async fn tc_048_2_cs() {
    run_in_local(conformance::tc_048_2_cs::run()).await;
}

#[tokio::test]
async fn tc_048_3_cs() {
    run_in_local(conformance::tc_048_3_cs::run()).await;
}

#[tokio::test]
async fn tc_051_cs() {
    run_in_local(conformance::tc_051_cs::run()).await;
}

#[tokio::test]
async fn tc_052_cs() {
    run_in_local(conformance::tc_052_cs::run()).await;
}

#[tokio::test]
async fn tc_053_cs() {
    run_in_local(conformance::tc_053_cs::run()).await;
}

#[tokio::test]
async fn tc_054_cs() {
    run_in_local(conformance::tc_054_cs::run()).await;
}

#[tokio::test]
async fn tc_055_cs() {
    run_in_local(conformance::tc_055_cs::run()).await;
}

#[tokio::test]
async fn tc_062_cs() {
    run_in_local(conformance::tc_062_cs::run()).await;
}
