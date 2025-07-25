FROM python:3.11-slim

# Install dependencies
RUN apt update && \
    apt install -y curl libglib2.0-0 ca-certificates && \
    curl -L https://github.com/Y2Z/monolith/releases/download/v2.10.1/monolith-gnu-linux-x86_64 \
         -o /usr/local/bin/monolith && \
    chmod +x /usr/local/bin/monolith

# Set work directory
WORKDIR /app

# Copy Python code
COPY app/ ./app/
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Create the saved_pages folder for mounting
RUN mkdir -p /app/saved_pages /app/data

# Run the server
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
