import os

# Ensure tests run with testing environment defaults
os.environ.setdefault("TESTING", "True")
os.environ.setdefault("DATABASE_URL", "sqlite:///:memory:")
