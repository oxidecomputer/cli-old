use anyhow::Result;
use cli_macro_impl::do_gen;
use quote::quote;

fn get_text(output: &proc_macro2::TokenStream) -> Result<String> {
    // Format the file with rustfmt.
    let content = rustfmt_wrapper::rustfmt(output).unwrap();

    // Add newlines after end-braces at <= two levels of indentation.
    Ok(if cfg!(not(windows)) {
        let regex = regex::Regex::new(r#"(})(\n\s{0,8}[^} ])"#).unwrap();
        regex.replace_all(&content, "$1\n$2").to_string()
    } else {
        let regex = regex::Regex::new(r#"(})(\r\n\s{0,8}[^} ])"#).unwrap();
        regex.replace_all(&content, "$1\r\n$2").to_string()
    })
}

#[test]
fn test_do_gen() {
    let mut actual = do_gen(
        quote! {
            tag = "disks",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {
                Attach(CmdDiskAttach),
                Create(CmdDiskCreate),
                Detach(CmdDiskDetach),
                Edit(CmdDiskEdit),
            }
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/disks.rs.gen", &get_text(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "organizations",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/organizations.rs.gen", &get_text(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "subnets",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/subnets.rs.gen", &get_text(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "routes",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/routes.rs.gen", &get_text(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "sleds",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/sleds.rs.gen", &get_text(&actual).unwrap());
}
