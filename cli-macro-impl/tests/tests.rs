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
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/organizations.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "subnets",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/subnets.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "routes",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/routes.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "sleds",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/sleds.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "instances",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/instances.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "vpcs",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/vpcs.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "projects",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/projects.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "images",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/images.rs.gen", &get_text_fmt(&actual).unwrap());

    actual = do_gen(
        quote! {
            tag = "images:global",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    )
    .unwrap();

    expectorate::assert_contents("tests/gen/images_global.rs.gen", &get_text_fmt(&actual).unwrap());
}
