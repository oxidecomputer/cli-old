use super::*;

#[test]
fn test_crud_gen() {
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
                View(CmdDiskView),
            }
        },
    );

    expectorate::assert_contents("gen/disks.rs", &actual.unwrap().to_string());

    actual = do_gen(
        quote! {
            tag = "organizations",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    );

    expectorate::assert_contents("gen/organizations.rs", &actual.unwrap().to_string());

    actual = do_gen(
        quote! {
            tag = "subnets",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    );

    expectorate::assert_contents("gen/subnets.rs", &actual.unwrap().to_string());
}
