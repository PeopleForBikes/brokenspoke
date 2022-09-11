from diagrams import Diagram
from diagrams.aws.compute import (
    ECR,
    Fargate,
    Lambda,
)
from diagrams.aws.storage import S3

with Diagram("Brochure ETL", show=False, direction="LR"):
    # images = ECR("ECR")
    svggloo = Lambda("SVGgloo")
    pdf_exporter = Fargate("PDF Exporter")
    (
        S3("City Ratings Store")
        >> Lambda("Rating to Shortcode")
        >> svggloo
        >> pdf_exporter
        >> Lambda("Add pages")
        >> Lambda("Bundle")
        >> S3("Brochure Store")
    )
    # images >> pdf_exporter
