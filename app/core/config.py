"""
Centralized configuration management with environment-specific settings
"""
import os
from typing import Optional, Dict, Any
from pydantic import BaseSettings, Field, validator
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()


class DatabaseConfig(BaseSettings):
    """Database configuration"""
    url: str = Field(
        default="postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db",
        env="DATABASE_URL"
    )
    track_modifications: bool = Field(default=False, env="SQLALCHEMY_TRACK_MODIFICATIONS")
    echo: bool = Field(default=False, env="SQLALCHEMY_ECHO")
    pool_size: int = Field(default=10, env="DATABASE_POOL_SIZE")
    max_overflow: int = Field(default=20, env="DATABASE_MAX_OVERFLOW")


class RedisConfig(BaseSettings):
    """Redis configuration for Celery"""
    url: str = Field(default="redis://localhost:6379/0", env="REDIS_URL")
    password: Optional[str] = Field(default=None, env="REDIS_PASSWORD")


class LLMConfig(BaseSettings):
    """LLM provider configurations"""
    openai_api_key: Optional[str] = Field(default=None, env="OPENAI_API_KEY")
    anthropic_api_key: Optional[str] = Field(default=None, env="ANTHROPIC_API_KEY")
    gemini_api_key: Optional[str] = Field(default=None, env="GEMINI_API_KEY")
    
    # Default models
    openai_default_model: str = Field(default="gpt-4", env="OPENAI_DEFAULT_MODEL")
    anthropic_default_model: str = Field(default="claude-3-5-sonnet-20241022", env="ANTHROPIC_DEFAULT_MODEL")
    gemini_default_model: str = Field(default="gemini-2.5-flash", env="GEMINI_DEFAULT_MODEL")


class ToolConfig(BaseSettings):
    """Tool configurations"""
    google_search_api_key: Optional[str] = Field(default=None, env="GOOGLE_SEARCH_API_KEY")
    google_search_engine_id: Optional[str] = Field(default=None, env="GOOGLE_SEARCH_ENGINE_ID")
    
    smtp_server: str = Field(default="smtp.gmail.com", env="SMTP_SERVER")
    smtp_port: int = Field(default=587, env="SMTP_PORT")
    smtp_username: Optional[str] = Field(default=None, env="SMTP_USERNAME")
    smtp_password: Optional[str] = Field(default=None, env="SMTP_PASSWORD")


class AgentConfig(BaseSettings):
    """Agent execution configuration"""
    max_iterations: int = Field(default=20, env="AGENT_MAX_ITERATIONS")
    default_timeout: int = Field(default=300, env="AGENT_DEFAULT_TIMEOUT")  # 5 minutes
    max_tokens: int = Field(default=1000, env="AGENT_MAX_TOKENS")
    temperature: float = Field(default=0.7, env="AGENT_TEMPERATURE")


class SecurityConfig(BaseSettings):
    """Security configuration"""
    secret_key: str = Field(default="dev-secret-key", env="SECRET_KEY")
    jwt_secret_key: Optional[str] = Field(default=None, env="JWT_SECRET_KEY")
    jwt_expiration_hours: int = Field(default=24, env="JWT_EXPIRATION_HOURS")


class LoggingConfig(BaseSettings):
    """Logging configuration"""
    level: str = Field(default="INFO", env="LOG_LEVEL")
    format: str = Field(
        default="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        env="LOG_FORMAT"
    )
    file_path: Optional[str] = Field(default=None, env="LOG_FILE_PATH")
    max_file_size: int = Field(default=10485760, env="LOG_MAX_FILE_SIZE")  # 10MB
    backup_count: int = Field(default=5, env="LOG_BACKUP_COUNT")


class AppConfig(BaseSettings):
    """Main application configuration"""
    # Environment
    environment: str = Field(default="development", env="ENVIRONMENT")
    debug: bool = Field(default=True, env="DEBUG")
    testing: bool = Field(default=False, env="TESTING")
    
    # Server
    host: str = Field(default="0.0.0.0", env="HOST")
    port: int = Field(default=5000, env="PORT")
    
    # Sub-configurations
    database: DatabaseConfig = DatabaseConfig()
    redis: RedisConfig = RedisConfig()
    llm: LLMConfig = LLMConfig()
    tools: ToolConfig = ToolConfig()
    agents: AgentConfig = AgentConfig()
    security: SecurityConfig = SecurityConfig()
    logging: LoggingConfig = LoggingConfig()
    
    @validator('environment')
    def validate_environment(cls, v):
        valid_environments = ['development', 'testing', 'production']
        if v not in valid_environments:
            raise ValueError(f'Environment must be one of: {valid_environments}')
        return v
    
    @property
    def is_development(self) -> bool:
        return self.environment == 'development'
    
    @property
    def is_testing(self) -> bool:
        return self.environment == 'testing'
    
    @property
    def is_production(self) -> bool:
        return self.environment == 'production'
    
    def get_database_url(self) -> str:
        """Get database URL with proper configuration"""
        return self.database.url
    
    def get_redis_url(self) -> str:
        """Get Redis URL with proper configuration"""
        return self.redis.url
    
    def get_llm_config(self, provider: str) -> Dict[str, Any]:
        """Get LLM configuration for a specific provider"""
        configs = {
            'openai': {
                'api_key': self.llm.openai_api_key,
                'default_model': self.llm.openai_default_model
            },
            'anthropic': {
                'api_key': self.llm.anthropic_api_key,
                'default_model': self.llm.anthropic_default_model
            },
            'gemini': {
                'api_key': self.llm.gemini_api_key,
                'default_model': self.llm.gemini_default_model
            }
        }
        return configs.get(provider, {})
    
    class Config:
        env_file = '.env'
        env_file_encoding = 'utf-8'


# Global configuration instance
config = AppConfig()


def get_config() -> AppConfig:
    """Get the global configuration instance"""
    return config


def reload_config() -> AppConfig:
    """Reload configuration (useful for testing)"""
    global config
    config = AppConfig()
    return config
