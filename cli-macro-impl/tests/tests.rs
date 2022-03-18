use cli_macro_impl::{do_gen, get_text_fmt};
use quote::quote;

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

    expectorate::assert_contents("tests/gen/disks.rs.gen", &get_text_fmt(&actual).unwrap());

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

    expectorate::assert_contents("tests/gen/organizations.rs.gen", &get_text_fmt(&actual).unwrap());

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

    expectorate::assert_contents("tests/gen/subnets.rs.gen", &get_text_fmt(&actual).unwrap());

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

    expectorate::assert_contents("tests/gen/routes.rs.gen", &get_text_fmt(&actual).unwrap());

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

    expectorate::assert_contents("tests/gen/sleds.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "instances",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/instances.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "vpcs",
        }
        .into(),
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        }
        .into(),
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/vpcs.rs.gen", &get_text_fmt(&actual).unwrap());
}
