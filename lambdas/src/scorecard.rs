use aws_lambda_events::event::sqs::SqsEvent;
use aws_sdk_s3::primitives::ByteStream;
use bnacore::combine::combine_mem;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use minijinja::Environment;
use serde_json::Value;
use usvg::{TreeParsing, TreeTextToPath};

const BUCKET_NAME: &str = "brokenspoke-analyzer";

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Load the parameters.
    let v: Value = serde_json::from_str(
        event.payload.records[0]
            .body
            .clone()
            .unwrap_or("{}".to_string())
            .as_str(),
    )?;

    // Compute the bucket key.
    let year = &v["year"];
    let country = &v["country"];
    let city = &v["city"];
    let region = v.get("region").unwrap_or(city);
    let key = format!("{}/{}-{}-{}.pdf", year, country, region, city);

    // Prepare the font database.
    let mut fontdb = usvg::fontdb::Database::new();
    dbg!(fontdb.len());
    // fontdb.load_system_fonts();
    // dbg!(fontdb.len());
    fontdb.load_fonts_dir("../assets/fonts");
    dbg!(fontdb.len());
    fontdb.load_font_file("../assets/fonts/DharmaGothicExtended copy/DharmaGothicE-Bold.otf")?;
    dbg!(fontdb.len());
    for face in fontdb.faces() {
        println!("{:#?}", face);
    }

    // Load the template.
    let source_page_1 = include_str!("../../assets/visuals/template-scorecard-pg1-v23.2.svg");
    let mut env = Environment::new();
    env.add_template("scorecard", source_page_1).unwrap();

    // Render the template to file for this specific record.
    let template = env.get_template("scorecard").unwrap();
    let rendered = template.render(&v)?;

    // Convert it to pdf.
    // let pdf_page_1 = svg2pdf::convert_str(source_page_1, svg2pdf::Options::default())?;
    let pdf_page_1 = pdf_convert(&rendered, &fontdb)?;
    std::fs::write("page_1.pdf", &pdf_page_1)?;

    // Load the second page and convert it to pdf.
    let source_page_2 = include_str!("../../assets/visuals/template-scorecard-pg2-v23.1.svg");
    // let pdf_page_2 = svg2pdf::convert_str(source_page_2, svg2pdf::Options::default())?;
    let pdf_page_2 = pdf_convert(source_page_2, &fontdb)?;
    std::fs::write("page_2.pdf", &pdf_page_2)?;

    // Combine the 2 pages.
    let full_pdf = vec![pdf_page_1.as_slice(), pdf_page_2.as_slice()];
    let mut scorecard = combine_mem(full_pdf.as_slice())?;
    let mut buffer: Vec<u8> = Vec::new();
    // let mut writer = BufWriter::new(buffer);
    scorecard.save_to(&mut buffer)?;
    scorecard.save("scorecard_test.pdf")?;

    // Upload to S3.
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let body = ByteStream::from(buffer);
    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(())
}

fn pdf_convert(svg: &str, fontdb: &usvg::fontdb::Database) -> Result<Vec<u8>, String> {
    let options = usvg::Options::default();

    let mut tree = usvg::Tree::from_str(svg, &options).map_err(|err| err.to_string())?;
    tree.convert_text(fontdb);

    let pdf = svg2pdf::convert_tree(&tree, Default::default());

    Ok(pdf)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_scorecard() {
        let id = "ID";
        let mut context = Context::default();
        context.request_id = id.to_string();

        let body = r#"{
          "ci": "Brussels",
          "co": "Belgium",
          "st": "BR",
          "uuid": "34330464-1e70-4988-9f92-52e3c6a869c0",
          "po": 188737,
          "ra": 83.87,
          "rasc": 84,
          "nw": 0,
          "aw": 0,
          "sf": 0,
          "rs": 0,
          "total": 0,
          "cssc": 0,
          "responses": 0,
          "nh": 81,
          "op": 87,
          "es": 88,
          "ret": 74,
          "rec": 86,
          "tr": 85,
          "bnasc": 84,
          "lsm": 482,
          "hsm": 88,
          "year": 2023,
          "country": "Belgium",
          "city": "Brussels",
          "region": "Brussels"
        }"#;

        let message = aws_lambda_events::sqs::SqsMessage {
            message_id: Some("MessageID_1".to_string()),
            receipt_handle: Some("ReceiptHandle".to_string()),
            body: Some(body.to_string()),
            md5_of_body: Some("fce0ea8dd236ccb3ed9b37dae260836f".to_string()),
            md5_of_message_attributes: None,
            attributes: HashMap::new(),
            message_attributes: HashMap::new(),
            event_source_arn: None,
            event_source: None,
            aws_region: None,
        };

        let payload = SqsEvent {
            records: vec![message],
        };

        let _event = LambdaEvent { payload, context };

        // let result = function_handler(event).await.unwrap();
    }
}
