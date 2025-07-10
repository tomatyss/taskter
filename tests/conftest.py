"""
Configuration and fixtures for pytest
"""
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
import os
import sys
from pathlib import Path

# Add project root to the Python path
BASE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(BASE_DIR))

from app.models.base import Base
from app.core.config import get_config as get_settings

# --- Database Fixtures ---

@pytest.fixture(scope="session")
def db_engine():
    """Fixture for creating a test database engine"""
    # Use an in-memory SQLite database for tests
    test_db_url = "sqlite:///:memory:"
    engine = create_engine(test_db_url, connect_args={"check_same_thread": False})
    
    # Create all tables
    Base.metadata.create_all(bind=engine)
    
    yield engine
    
    # Teardown: drop all tables
    Base.metadata.drop_all(bind=engine)

@pytest.fixture(scope="function")
def db_session(db_engine):
    """Fixture for creating a test database session for each test function"""
    connection = db_engine.connect()
    
    # Begin a non-ORM transaction
    transaction = connection.begin()
    
    # Bind an individual session to the connection
    Session = sessionmaker(autocommit=False, autoflush=False, bind=connection)
    session = Session()
    
    yield session
    
    # Rollback the transaction and close the connection
    session.close()
    transaction.rollback()
    connection.close()

# --- Settings Fixture ---

@pytest.fixture(scope="session")
def test_settings():
    """Fixture for providing test settings"""
    # Override settings for testing environment
    os.environ["ENVIRONMENT"] = "test"
    return get_settings()
