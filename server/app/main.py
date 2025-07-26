from fastapi import FastAPI, HTTPException, Path, Depends, Query
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
from sqlalchemy.orm import Session
from sqlalchemy import func
from typing import List, Optional
import os

from . import database, monolith_handler

app = FastAPI()


class URLRequest(BaseModel):
    url: str
    tags: Optional[List[str]] = []


def get_db():
    """Dependency to get database session."""
    db = database.SessionLocal()
    try:
        yield db
    finally:
        db.close()


@app.post("/save/")
def save_url(data: URLRequest, db: Session = Depends(get_db)):
    """Save a URL using monolith and store metadata in database."""
    try:
        path = monolith_handler.save_with_monolith(data.url, "./saved_pages")
        title, content = monolith_handler.extract_text_and_title(data.url)

        site = database.SiteData(
            url=data.url,
            title=title,
            tags=",".join(data.tags),
            content=content,
            saved_path=path
        )
        db.add(site)
        db.commit()
        db.refresh(site)
        return {"status": "ok", "id": site.id}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/page/{id}", response_class=HTMLResponse)
def get_saved_page(
    id: int = Path(..., description="The ID of the saved page"),
    db: Session = Depends(get_db)
):
    """Retrieve and serve a saved HTML page by ID."""
    page = db.query(database.SiteData).filter(database.SiteData.id == id).first()
    if not page:
        raise HTTPException(status_code=404, detail="Page not found")
    
    if not os.path.exists(page.saved_path):
        raise HTTPException(status_code=500, detail="Saved HTML file not found")

    with open(page.saved_path, "r", encoding="utf-8") as f:
        html_content = f.read()

    return HTMLResponse(content=html_content, status_code=200)


@app.get("/search_text")
def search_text(
    q: str = Query(..., description="Search query"),
    limit: int = Query(5, ge=1, le=100, description="Number of results to return"),
    db: Session = Depends(get_db)
):
    """
    Search for sites by text content using ripgrep-like fuzzy matching.
    All search terms must be present in the content (AND logic).
    """
    search_terms = q.strip().split()
    
    if not search_terms:
        return []
    
    # Build query with case-insensitive LIKE for each term
    query = db.query(database.SiteData)
    
    for term in search_terms:
        query = query.filter(
            func.lower(database.SiteData.content).like(f"%{term.lower()}%")
        )
    
    # Order by creation date (newest first) and limit results
    results = query.order_by(database.SiteData.created_at.desc()).limit(limit).all()
    
    return results