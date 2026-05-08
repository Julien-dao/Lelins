from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")

    mistral_api_key: str = ""
    mistral_model: str = "mistral-small-latest"
    default_brand: str = "lelins"
    app_host: str = "127.0.0.1"
    app_port: int = 8000


@lru_cache
def get_settings() -> Settings:
    return Settings()
