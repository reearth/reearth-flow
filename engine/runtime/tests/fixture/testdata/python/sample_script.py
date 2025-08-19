# Sample Python script for testing
# This script receives attributes as a dictionary and modifies them

# The 'attributes' variable is pre-populated with input data

# Example transformations
if 'name' in attributes:
    attributes['name'] = attributes['name'].upper()

if 'value' in attributes:
    attributes['value'] = attributes['value'] * 10

# Add timestamp
from datetime import datetime
attributes['processed_at'] = datetime.now().isoformat()

# Add a calculation
attributes['magic_number'] = 42

# The modified 'attributes' dictionary will be returned as output