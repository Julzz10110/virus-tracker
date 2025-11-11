from __future__ import annotations

from typing import Dict, Optional

import requests

from .schemas import VariantPayload


class BackendClient:
    """HTTP client for interacting with the Rust backend API."""

    def __init__(self, base_url: str, timeout_seconds: float = 10.0) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout_seconds
        self._session = requests.Session()

    def _url(self, path: str) -> str:
        if not path.startswith("/"):
            path = f"/{path}"
        return f"{self.base_url}{path}"

    def health_check(self) -> bool:
        try:
            response = self._session.get(self._url("/health"), timeout=self.timeout)
            response.raise_for_status()
            try:
                payload = response.json()
            except ValueError:
                payload = response.text
            if isinstance(payload, dict):
                return payload.get("status") == "healthy"
            return str(payload).strip().upper() == "OK"
        except Exception:
            return False

    def update_reference(self, reference: str) -> Dict:
        payload = {"reference": reference}
        response = self._session.post(
            self._url("/reference"), json=payload, timeout=self.timeout
        )
        response.raise_for_status()
        return response.json()

    def upload_variant(self, variant: VariantPayload) -> Dict:
        response = self._session.post(
            self._url("/upload"), json=variant.dict(), timeout=self.timeout
        )
        response.raise_for_status()
        return response.json()

    def fetch_stats(self) -> Optional[Dict]:
        response = self._session.get(
            self._url("/variants/stats"), timeout=self.timeout
        )
        if response.ok:
            return response.json()
        return None

    def fetch_variants(self, params: Optional[Dict] = None):
        response = self._session.get(
            self._url("/variants"),
            timeout=self.timeout,
            params=params or {},
        )
        response.raise_for_status()
        return response.json()

