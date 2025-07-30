from fastapi import FastAPI, HTTPException, Path, Depends, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
from sqlalchemy.orm import Session
from sqlalchemy import func, or_
from typing import List, Optional
import os

from . import database, monolith_handler

app = FastAPI()

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "*",  # For development
        # "chrome-extension://your-extension-id-here",  # For production
        # "moz-extension://your-extension-id-here",     # For Firefox
    ],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
)

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
    tags: Optional[List[str]] = Query(None, description="Filter by tags (comma-separated)"),
    whole_word: bool = Query(False, description="Search for whole words only (not part of other words)"),
    db: Session = Depends(get_db)
):
    """
    Search for sites by exact phrase matching in text content.
    Searches for the complete query string as a phrase.
    Optionally filter by tags.
    """
    search_phrase = q.strip()
    
    if not search_phrase:
        return []
    
    if whole_word:
        # SQLite word boundary simulation using LIKE patterns
        # Check for word boundaries before AND after the search term
        search_lower = search_phrase.lower()
        
        # Define word boundary characters (non-alphanumeric)
        boundaries = [' ', '.', ',', ';', ':', '!', '?', '\n', '\t', '(', ')', '[', ']', '{', '}', '"', "'"]
        
        conditions = []
        
        # Word at the very beginning of text (start + boundary after)
        for boundary in boundaries:
            conditions.append(func.lower(database.SiteData.content).like(f"{search_lower}{boundary}%"))
        
        # Word in the middle (boundary before + word + boundary after)
        for bound_before in boundaries:
            for bound_after in boundaries:
                conditions.append(func.lower(database.SiteData.content).like(f"%{bound_before}{search_lower}{bound_after}%"))
        
        # Word at the very end of text (boundary before + end)
        for boundary in boundaries:
            conditions.append(func.lower(database.SiteData.content).like(f"%{boundary}{search_lower}"))
        
        # Exact match (whole content is just the word)
        conditions.append(func.lower(database.SiteData.content) == search_lower)
        
        query = db.query(database.SiteData).filter(or_(*conditions))
    else:
        # Search for the exact phrase (case-insensitive)
        query = db.query(database.SiteData).filter(
            func.lower(database.SiteData.content).like(f"%{search_phrase.lower()}%")
        )
    
    # Filter by tags when provided
    if tags:
        for tag in tags:
            query = query.filter(
                func.lower(database.SiteData.tags).like(f"%{tag.lower()}%")
            )

    # Order by creation date (newest first) and limit results
    results = query.order_by(database.SiteData.created_at.desc()).limit(limit).all()
    
    return results


@app.get("/search_title")
def search_text(
    q: str = Query(..., description="Search query"),
    limit: int = Query(5, ge=1, le=100, description="Number of results to return"),
    tags: Optional[List[str]] = Query(None, description="Filter by tags (comma-separated)"),
    whole_word: bool = Query(False, description="Search for whole words only (not part of other words)"),
    db: Session = Depends(get_db)
):
    """
    Search for sites by exact phrase matching in their titles.
    Searches for the complete query string as a phrase.
    Optionally filter by tags.
    """
    search_phrase = q.strip()
    
    if not search_phrase:
        return []
    
    if whole_word:
        # SQLite word boundary simulation using LIKE patterns
        # Check for word boundaries before AND after the search term
        search_lower = search_phrase.lower()
        
        # Define word boundary characters (non-alphanumeric)
        boundaries = [' ', '.', ',', ';', ':', '!', '?', '\n', '\t', '(', ')', '[', ']', '{', '}', '"', "'"]
        
        conditions = []
        
        # Word at the very beginning of text (start + boundary after)
        for boundary in boundaries:
            conditions.append(func.lower(database.SiteData.title).like(f"{search_lower}{boundary}%"))
        
        # Word in the middle (boundary before + word + boundary after)
        for bound_before in boundaries:
            for bound_after in boundaries:
                conditions.append(func.lower(database.SiteData.title).like(f"%{bound_before}{search_lower}{bound_after}%"))
        
        # Word at the very end of text (boundary before + end)
        for boundary in boundaries:
            conditions.append(func.lower(database.SiteData.title).like(f"%{boundary}{search_lower}"))
        
        # Exact match (whole title is just the word)
        conditions.append(func.lower(database.SiteData.title) == search_lower)
        
        query = db.query(database.SiteData).filter(or_(*conditions))
    else:
        # Search for the exact phrase (case-insensitive)
        query = db.query(database.SiteData).filter(
            func.lower(database.SiteData.title).like(f"%{search_phrase.lower()}%")
        )
    
    # Filter by tags when provided
    if tags:
        for tag in tags:
            query = query.filter(
                func.lower(database.SiteData.tags).like(f"%{tag.lower()}%")
            )

    # Order by creation date (newest first) and limit results
    results = query.order_by(database.SiteData.created_at.desc()).limit(limit).all()
    
    return results


@app.get("/get_url")
def get_url(id: int = Query(..., description="Id of the URL to retrieve"), db: Session = Depends(get_db)):
    """
    Retrieve the original url of the entry matching the provided id 
    """

    url = db.query(database.SiteData.url).where(database.SiteData.id == id).scalar()

    if url is None:
        raise HTTPException(status_code=404, detail="ID not found in db")

    return {"url": url}
    
