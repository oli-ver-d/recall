import subprocess
from pathlib import Path
from bs4 import BeautifulSoup
import requests
import tempfile

def save_with_monolith(url: str, output_dir: str = "/app/saved_pages") -> str:
    output_path = Path(output_dir) / f"{hash(url)}.html"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["monolith", url, "-o", str(output_path)], check=True)
    return str(output_path)

def extract_text_and_title(url: str) -> tuple[str, str]:
    r = requests.get(url)
    soup = BeautifulSoup(r.text, "html.parser")
    title = soup.title.string.strip() if soup.title else "No title"
    text = soup.get_text()
    return title, text
