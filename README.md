# Recall - Web Page Archiver & Search

Recall is a web page archiving and search tool that saves web pages locally and makes them searchable. It consists of a FastAPI backend that archives pages using [monolith](https://github.com/Y2Z/monolith) and a Rust CLI for saving and searching pages.

## Features

- **Archive web pages** as single HTML files using monolith
- **Full-text search** through archived content with exact phrase matching
- **Tag support** for organizing saved pages - NOT COMPLETE, can set tags but are currently unused
- **Fast CLI interface** for saving and searching
- **Docker deployment** for easy setup
- **SQLite database** for metadata storage

## Architecture

- **Backend**: Python FastAPI server running in Docker with SQLite database
- **Frontend**: Rust CLI tool for interacting with the API
- **Storage**: Web pages saved as single HTML files via monolith
- **Search**: Exact phrase matching in page content

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust (for building the CLI tool)
- Git
- Mask

### 1. Clone and Setup Backend

```bash
git clone https://github.com/oli-ver-d/recall.git
cd recall
cd server

# Build and start the Docker container
docker build -t recall .
docker run -i -p 8000:8000 -v "$(pwd)/saved_pages:/app/saved_pages" -v "$(pwd)/data:/app/data" recall

# The API will be available at http://localhost:8000
```

### 2. Build the CLI Tool

```bash
cd cli
cargo build --release

# The binary will be at target/release/recall
# Optionally, install it to your PATH:
cargo install --path .
```

### 3. Test the Setup

```bash
# Save a web page
recall save "https://example.com" --tags example test

# Search for content
recall search "example"
```

## Usage

### CLI Commands

```bash
Archive and search web pages

Usage: recall [OPTIONS] <COMMAND>

Commands:
  search  Search through saved web pages
  save    Save a URL to the archive
  help    Print this message or the help of the given subcommand(s)

Options:
  -s, --server <SERVER>  Server URL [env: RECALL_SERVER=] [default: http://localhost:8000]
  -h, --help             Print help
  -V, --version          Print version

Search through saved web pages

Usage: recall search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Search query string

Options:
  -l, --limit <LIMIT>  Maximum number of results to return [default: 5]
  -t, --tags <TAGS>    Tags to filter the search by
  -h, --help           Print help

Save a URL to the archive

Usage: recall save [OPTIONS] <URL>

Arguments:
  <URL>  URL to save

Options:
  -t, --tags <TAGS>  Tags to add to the saved page
  -h, --help         Print help
```

#### Save a Web Page
```bash
# Basic save
recall save "https://docs.python.org"

# Save with tags
recall save "https://docs.python.org" --tags python --tags documentation

# Use custom server
recall --server http://localhost:8000 save "https://example.com"
```

#### Search Pages
```bash
# Basic search (exact phrase matching)
recall search "python tutorial"

# Limit results
recall search "docker" --limit 10

# Use custom server
recall --server http://your-server:8000 search "fastapi"
```

### API Endpoints

The FastAPI backend provides these endpoints:

- `POST /save/` - Save a URL with optional tags
- `GET /search_text?q=query&limit=5` - Search through page content
- `GET /page/{id}` - Retrieve a saved HTML page by ID

