#[derive(Debug)]
pub struct AWSConfig {
    pub config: aws_types::config::Config,
}

impl AWSConfig {
    pub fn set_config(config: aws_types::config::Config) -> Self {
        Self { config }
    }

    pub fn migration_init(&self) -> AWSClient {
        let aws_client = AWSClient {
            config: self.get_config(),
            s3_client: self.s3_client(),
            sqs_client: self.sqs_client(),
        };

        aws_client
    }

    fn get_config(&self) -> aws_types::config::Config {
        self.config.clone()
    }

    fn s3_client(&self) -> aws_sdk_s3::Client {
        aws_sdk_s3::Client::new(&self.config)
    }

    fn sqs_client(&self) -> aws_sdk_sqs::Client {
        aws_sdk_sqs::Client::new(&self.config)
    }
}

#[derive(Debug)]
pub struct AWSClient {
    pub config: aws_types::config::Config,
    pub s3_client: aws_sdk_s3::Client,
    pub sqs_client: aws_sdk_sqs::Client,
}
