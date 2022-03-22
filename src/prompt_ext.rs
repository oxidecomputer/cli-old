use std::str::FromStr;

use anyhow::Result;

trait PromptExt {
    fn prompt() -> Result<Self>
    where
        Self: Sized;
}

impl PromptExt for oxide_api::types::RouteDestination {
    fn prompt() -> Result<Self> {
        let route_destination_type = oxide_api::types::RouteDestinationType::prompt()?;

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
            oxide_api::types::RouteDestinationType::Ipnet => {
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
    fn prompt() -> Result<Self> {
        let items = oxide_api::types::RouteDestination::variants();

        let index = dialoguer::Select::new()
            .with_prompt("Select a route destination type:")
            .items(&items[..])
            .interact();

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
    fn prompt() -> Result<Self> {
        let route_target_type = oxide_api::types::RouteTargetType::prompt()?;

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
            oxide_api::types::RouteTargetType::Internetgateway => oxide_api::types::RouteTarget::InternetGateway(value),
        })
    }
}

impl PromptExt for oxide_api::types::RouteTargetType {
    fn prompt() -> Result<Self> {
        let items = oxide_api::types::RouteTarget::variants();

        let index = dialoguer::Select::new()
            .with_prompt("Select a route target type:")
            .items(&items[..])
            .interact();

        let item = match index {
            Ok(i) => items[i].to_string(),
            Err(err) => {
                anyhow::bail!("prompt failed: {}", err);
            }
        };

        oxide_api::types::RouteTargetType::from_str(&item)
    }
}