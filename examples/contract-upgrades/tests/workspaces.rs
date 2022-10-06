#[tokio::test]
async fn test_upgrade() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(&workspaces::compile_project("./upgrade-a").await?)
        .await?;

    assert!(contract
        .call("some_new_function")
        .transact()
        .await?
        .is_failure());

    let res = contract
        .call("upgrade")
        .args(workspaces::compile_project("./upgrade-b").await?)
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert_eq!(res.logs()[0], "performing arbitrary migration logic");

    let res = contract.call("some_new_function").transact().await?;
    assert_eq!(res.logs()[0], "can call some new function now!");
    assert!(res.is_success());

    Ok(())
}
