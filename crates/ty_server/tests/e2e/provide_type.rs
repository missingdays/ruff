use lsp_types::{Position, Range};
use ruff_db::system::SystemPath;
use ty_server::ClientOptions;
use crate::TestServerBuilder;

#[test]
fn provide_str_type() -> anyhow::Result<()> {
    let workspace_root = SystemPath::new("src");
    let foo = SystemPath::new("src/foo.py");
    let foo_content = "\
class C:
    pass
def foo() -> C:
    return C()
";

    let mut server = TestServerBuilder::new()?
        .with_workspace(workspace_root, ClientOptions::default())?
        .with_file(foo, foo_content)?
        .enable_pull_diagnostics(true)
        .build()?
        .wait_until_workspaces_are_initialized()?;

    server.open_text_document(foo, &foo_content, 1);
    let provide_type_response = server.provide_type_request(
        foo,
        Range::new(
            Position::new(3, 11),
            Position::new(3, 14)
        )
    )?;

    insta::assert_debug_snapshot!(provide_type_response);

    Ok(())
}