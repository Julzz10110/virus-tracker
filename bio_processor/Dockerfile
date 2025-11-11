FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . /app/bio_processor
ENV PYTHONPATH=/app

CMD ["uvicorn", "bio_processor.main:app", "--host", "0.0.0.0", "--port", "8000"]