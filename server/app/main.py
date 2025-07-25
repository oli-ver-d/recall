from fastapi import FastAPI, HTTPException, Path
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
import os
from typing import List, Optional
from . import database, monolith_handler

app = FastAPI()

class URLRequest(BaseModel):
    url: str
    tags: Optional[List[str]] = []

@app.post("/save/")
def save_url(data: URLRequest):
    db = database.SessionLocal()
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
    finally:
        db.close()

@app.get("/page/{id}", response_class=HTMLResponse)
def get_saved_page(id: int = Path(..., description="The ID of the saved page")):
    db = database.SessionLocal()
    try:
        page = db.query(database.SiteData).filter(database.SiteData.id == id).first()
        if not page:
            raise HTTPException(status_code=404, detail="Page not found")
        
        # Read the saved HTML file
        if not os.path.exists(page.saved_path):
            raise HTTPException(status_code=500, detail="Saved HTML file not found")

        with open(page.saved_path, "r", encoding="utf-8") as f:
            html_content = f.read()

        return HTMLResponse(content=html_content, status_code=200)

    finally:
        db.close()