use std::str::FromStr;

use anyhow::Result;

pub trait PromptExt {
    fn prompt(base: &str) -> Result<Self>
    where
        Self: Sized;
}

impl PromptExt for oxide_api::types::RouteDestination {
    fn prompt(base: &str) -> Result<Self> {
        let route_destination_type = oxide_api::types::RouteDestinationType::prompt(base)?;

        let value: String = match dialoguer::Input::<String>::new()
            .with_prompt(&format!("{} value?", route_destination_type))
            .interact_text()
        {
            Ok(i) => i,
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };

        Ok(match route_destination_type {
            oxide_api::types::RouteDestinationType::Ip => oxide_api::types::RouteDestination::Ip(value),
            oxide_api::types::RouteDestinationType::IpNet => {
                let ipnet = oxide_api::types::IpNet::from_str(&value)
                    .map_err(|e| anyhow::anyhow!("invalid ipnet {}: {}", value, e));

                oxide_api::types::RouteDestination::IpNet(ipnet?)
            }
            oxide_api::types::RouteDestinationType::Vpc => oxide_api::types::RouteDestination::Vpc(value),
            oxide_api::types::RouteDestinationType::Subnet => oxide_api::types::RouteDestination::Subnet(value),
        })
    }
}

impl PromptExt for oxide_api::types::RouteDestinationType {
    fn prompt(base: &str) -> Result<Self> {
        let items = oxide_api::types::RouteDestination::variants();

        let index = dialoguer::Select::new().with_prompt(base).items(&items[..]).interact();

        let item = match index {
            Ok(i) => items[i].to_string(),
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };

        oxide_api::types::RouteDestinationType::from_str(&item)
    }
}

impl PromptExt for oxide_api::types::RouteTarget {
    fn prompt(base: &str) -> Result<Self> {
        let route_target_type = oxide_api::types::RouteTargetType::prompt(base)?;

        let value: String = match dialoguer::Input::<String>::new()
            .with_prompt(&format!("{} value?", route_target_type))
            .interact_text()
        {
            Ok(i) => i,
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };

        Ok(match route_target_type {
            oxide_api::types::RouteTargetType::Ip => oxide_api::types::RouteTarget::Ip(value),
            oxide_api::types::RouteTargetType::Vpc => oxide_api::types::RouteTarget::Vpc(value),
            oxide_api::types::RouteTargetType::Subnet => oxide_api::types::RouteTarget::Subnet(value),
            oxide_api::types::RouteTargetType::Instance => oxide_api::types::RouteTarget::Instance(value),
            oxide_api::types::RouteTargetType::InternetGateway => oxide_api::types::RouteTarget::InternetGateway(value),
        })
    }
}

impl PromptExt for oxide_api::types::RouteTargetType {
    fn prompt(base: &str) -> Result<Self> {
        let items = oxide_api::types::RouteTarget::variants();

        let index = dialoguer::Select::new().with_prompt(base).items(&items[..]).interact();

        let item = match index {
            Ok(i) => items[i].to_string(),
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };

        oxide_api::types::RouteTargetType::from_str(&item)
    }
}

impl PromptExt for oxide_api::types::Ipv4Net {
    fn prompt(base: &str) -> Result<Self> {
        let input = dialoguer::Input::<String>::new()
            .with_prompt(base)
            .validate_with(|input: &String| -> Result<(), &str> {
                let ipnet = oxide_api::types::Ipv4Net::from_str(input);

                if ipnet.is_err() {
                    Err("invalid IPv4 network")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        oxide_api::types::Ipv4Net::from_str(&input).map_err(|e| anyhow::anyhow!("invalid ipv4net `{}`: {}", input, e))
    }
}

impl PromptExt for oxide_api::types::Ipv6Net {
    fn prompt(base: &str) -> Result<Self> {
        let input = dialoguer::Input::<String>::new()
            .with_prompt(base)
            .validate_with(|input: &String| -> Result<(), &str> {
                let ipnet = oxide_api::types::Ipv6Net::from_str(input);

                if ipnet.is_err() {
                    Err("invalid IPv6 network")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        oxide_api::types::Ipv6Net::from_str(&input).map_err(|e| anyhow::anyhow!("invalid ipv6net `{}`: {}", input, e))
    }
}

impl PromptExt for oxide_api::types::ByteCount {
    fn prompt(base: &str) -> Result<Self> {
        let input = dialoguer::Input::<String>::new().with_prompt(base).interact_text()?;
        // Echo the user's input, and print in a normalized base-2 form,
        // to give them the chance to verify their input.
        let bytes = input.parse::<::byte_unit::Byte>()?;
        println!("Using {} bytes ({})", bytes, bytes.get_appropriate_unit(true));
        Ok(oxide_api::types::ByteCount::try_from(bytes.get_bytes())?)
    }
}

impl PromptExt for oxide_api::types::ImageSource {
    fn prompt(base: &str) -> Result<Self> {
        let input = dialoguer::Input::<String>::new().with_prompt(base).interact_text()?;

        oxide_api::types::ImageSource::from_str(&input)
    }
}

impl PromptExt for oxide_api::types::DiskSource {
    fn prompt(base: &str) -> Result<Self> {
        let disk_source_type = oxide_api::types::DiskSourceType::prompt(base)?;

        fn get_value(prompt: &str) -> Result<String> {
            Ok(dialoguer::Input::<String>::new().with_prompt(prompt).interact_text()?)
        }

        Ok(match disk_source_type {
            oxide_api::types::DiskSourceType::Blank => {
                let input = get_value("Block size")?;
                let bytes = input.parse::<::byte_unit::Byte>()?;
                println!("Using {} bytes ({})", bytes, bytes.get_appropriate_unit(true));
                let byte_count = oxide_api::types::ByteCount::try_from(bytes.get_bytes())?;
                oxide_api::types::DiskSource::Blank(byte_count as i64)
            }
            oxide_api::types::DiskSourceType::Image => oxide_api::types::DiskSource::Image(get_value("Image ID")?),
            oxide_api::types::DiskSourceType::GlobalImage => {
                oxide_api::types::DiskSource::GlobalImage(get_value("Global image ID")?)
            }
            oxide_api::types::DiskSourceType::Snapshot => {
                oxide_api::types::DiskSource::Snapshot(get_value("Snapshot ID")?)
            }
        })
    }
}

impl PromptExt for oxide_api::types::DiskSourceType {
    fn prompt(base: &str) -> Result<Self> {
        let items = oxide_api::types::DiskSource::variants();
        let index = dialoguer::Select::new().with_prompt(base).items(&items[..]).interact();
        let item = match index {
            Ok(i) => items[i].to_string(),
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };
        oxide_api::types::DiskSourceType::from_str(&item)
    }
}
