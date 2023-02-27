use tokio_stream::StreamExt;

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

#[derive(Debug)]
pub struct CSVParser {}

impl CSVParser {
    pub async fn from_s3<T: serde::de::DeserializeOwned>(csv: CSV) -> Result<Vec<T>, E> {
        println!("downloading {:?} from {:?}", csv.key, csv.bucket_name);
        let stream = csv
            .aws_client
            .get_object()
            .bucket(csv.bucket_name)
            .key(csv.key)
            .send()
            .await?
            .body
            .into_async_read();

        println!("reading");
        // Create a CSV reader
        let mut csv_reader = csv_async::AsyncReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .double_quote(false)
            .escape(Some(b'\\'))
            .flexible(true)
            .create_deserializer(stream);

        // Iterate over the CSV rows
        let mut records = csv_reader.deserialize::<_>();
        let mut rows = Vec::new();
        while let Some(record) = records.next().await {
            let row: T = record?;
            rows.push(row);
        }
        println!("finished");
        Ok(rows)
    }
}

#[derive(Debug)]
pub struct CSV {
    pub aws_client: aws_sdk_s3::Client,
    pub bucket_name: String,
    pub key: String,
}

#[derive(Debug)]
pub struct CSVBuilder {
    aws_client: aws_sdk_s3::Client,
    bucket_name: Option<String>,
    key: Option<String>,
}

impl CSVBuilder {
    pub fn new(aws_client: &aws_sdk_s3::Client) -> CSVBuilder {
        Self {
            aws_client: aws_client.clone(),
            bucket_name: None,
            key: None,
        }
    }

    pub fn bucket(mut self, name: String) -> CSVBuilder {
        self.bucket_name = Some(name);
        self
    }

    pub fn key(mut self, name: String) -> CSVBuilder {
        self.key = Some(name);
        self
    }

    pub fn build(self) -> CSV {
        CSV {
            aws_client: self.aws_client,
            bucket_name: self.bucket_name.expect("BUCKET_NAME bucket must be set"),
            key: self.key.expect("KEY bucket must be set"),
        }
    }
}
