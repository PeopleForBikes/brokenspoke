from diagrams import Diagram
from diagrams.aws.storage import S3
from diagrams.aws.compute import Lambda, Fargate, ECR

with Diagram("Brochure ETL", show=False, direction="LR"):
    images = ECR("ECR")
    svggloo = Fargate("SVGgloo")
    S3("City Ratings Store") >> Lambda("Rating to Shortcode") >> svggloo >> S3("Brochure Store")
    images >> svggloo
