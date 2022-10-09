from diagrams import Diagram
from diagrams.aws.compute import Lambda
from diagrams.aws.storage import S3

with Diagram("Retrieve ETL", show=False, direction="LR"):
    (
        S3("City Ratings Store")
        >> Lambda("Retrieve")
        >> Lambda("Bundle")
        >> S3("Brochure Store")
    )
