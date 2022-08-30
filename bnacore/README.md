# bnacore

Contains the common pieces between the different BNA projects.

## Python bindings

This crate also generates Python bindings for the models, as well as a few
utility functions.

The `justfile` contains the tasks to package the bindings or to install them
locally in developer mode.

### Example

```py
import bnacore
from pathlib import Path

csv_file = Path("../retrieve/examples/city_ratings_2021_v15.csv")
recs = pfbcore.ScoreCard.load_csv(str(csv_file))
rec = recs[0]
rec.city.name
```
