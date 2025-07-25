from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
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
