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
        .with_workspace(workspace_root, Some(ClientOptions::default()))?
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

#[test]
fn provide_nested_class_type() -> anyhow::Result<()> {
    let workspace_root = SystemPath::new("src");
    let foo = SystemPath::new("src/foo.py");
    let foo_content = "\
class A:
    class B:
        pass

b = A.B()
b
";

    let mut server = TestServerBuilder::new()?
        .with_workspace(workspace_root, Some(ClientOptions::default()))?
        .with_file(foo, foo_content)?
        .enable_pull_diagnostics(true)
        .build()?
        .wait_until_workspaces_are_initialized()?;

    server.open_text_document(foo, &foo_content, 1);
    let provide_type_response = server.provide_type_request(
        foo,
        Range::new(
            Position::new(5, 0),
            Position::new(5, 1)
        )
    )?;

    insta::assert_debug_snapshot!(provide_type_response);

    Ok(())
}


#[test]
fn provide_generic_class_type() -> anyhow::Result<()> {
    let workspace_root = SystemPath::new("src");
    let foo = SystemPath::new("src/foo.py");
    let foo_content = "\
class A[T]:
    i: T
    def __init__(self, i: T):
        self.i = i

a = A(1)
a
";

    let mut server = TestServerBuilder::new()?
        .with_workspace(workspace_root, Some(ClientOptions::default()))?
        .with_file(foo, foo_content)?
        .enable_pull_diagnostics(true)
        .build()?
        .wait_until_workspaces_are_initialized()?;

    server.open_text_document(foo, &foo_content, 1);
    let provide_type_response = server.provide_type_request(
        foo,
        Range::new(
            Position::new(6, 0),
            Position::new(6, 1)
        )
    )?;

    insta::assert_debug_snapshot!(provide_type_response);

    Ok(())
}

