# Serverless Geo Location CSV variant

[MaxMind](https://dev.maxmind.com/) provides IP intelligence through the GeoIP brand. Over 5,000 companies use GeoIP data to locate their Internet visitors, show them relevant content and ads, perform analytics, enforce digital rights, and efficiently route Internet traffic.
[GeoIP2 IP](https://dev.maxmind.com/geoip?) intelligence products and services can provide data on geolocation, network information, anonymizer status and offer three types of data

- Binary Database
- CSV Database
- WebService

This repository is how to import the CSV Databases into DynamoDB. To do such operation and this will result in a much slower results than using the [Binary version](https://github.com/ymwjbxxq/rust_geoip) solution.

At higher level the steps are;

- Copy the files into the Amazon S3
- Run the Migration Lambda function
- The Migration Lambda function will merge the CSV in one object and send it to an Amazon SQS
- Amazon SQS will trigger the Import Lambda function
- The Import Lambda function will insert into Amazon DynamoDB

## Requirements

* [Create an AWS account](https://portal.aws.amazon.com/gp/aws/developer/registration/index.html) if you do not already have one and log in. The IAM user that you use must have sufficient permissions to make necessary AWS service calls and manage AWS resources.
* [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html) installed and configured
* [Git Installed](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
* [AWS Serverless Application Model](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html) (AWS SAM) installed
* [Rust](https://www.rust-lang.org/) 1.56.0 or higher
* [cargo-zigbuild](https://github.com/messense/cargo-zigbuild) and [Zig](https://ziglang.org/) for cross-compilation

## Deployment Instructions

The SAM template deploys an Amazon S3, an Amazon API Gateway HTTP API, three AWS Lambda function and two tables in Amazon DynamoDB.

1. Create a new directory, navigate to that directory in a terminal and clone the GitHub repository:
    ``` 
    git clone https://github.com/ymwjbxxq/rust_geoip_csv_variant
    ```
2. Change directory to the pattern directory:
    ```
    cd rust_geoip_csv_variant
    ```
3. Install dependencies and build:
    ```
    make build
    ```
4. From the command line, use AWS SAM to deploy the AWS resources for the pattern as specified in the template.yml file:
    ```
    make deploy
    ```
5. During the prompts:
    * Enter a stack name
    * Enter the desired AWS Region
    * Allow SAM CLI to create IAM roles with the required permissions.

    Once you have run `sam deploy -guided` mode once and saved arguments to a configuration file (samconfig.toml), you can use `sam deploy` in future to use these defaults.

6. Note the outputs from the SAM deployment process. These contain the resource names and ARNs which are used for testing.

### Testing

**CSV files are not provided**

Once the application is deployed, 

1. Retrieve the the Amazon S3 Bucket name value from CloudFormation Outputs and copy the csv files into the bucket.
2. Run the Migration Lambda manually from the console
3. Retrieve the HttpApiEndpoint value from CloudFormation Outputs. Then, either browse to the endpoint in a web browser or call the endpoint from Postman.

Example GET Request: https://{HttpApiId}.execute-api.{region}.amazonaws.com/countrycode

Response:
```
{
  "countrycode": "IT"
}
```

## Cleanup
 
1. Delete the stack
    ```bash
    make delete
    ```
----
