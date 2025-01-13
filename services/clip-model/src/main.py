from fastapi import FastAPI, HTTPException, Body
from pydantic import BaseModel
import uvicorn
import torch
from PIL import Image
import base64
import io
from transformers import CLIPModel, CLIPProcessor
from typing import Dict
import numpy as np
from contextlib import asynccontextmanager

# Global variables for model and processor
model = None
processor = None

def init_model():
    """Initialize the CLIP model and processor"""
    global model, processor
    model = CLIPModel.from_pretrained("openai/clip-vit-base-patch32")
    processor = CLIPProcessor.from_pretrained("openai/clip-vit-base-patch32")
    model.eval()  # Set model to evaluation mode

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Load the ML model
    init_model()
    yield
    # Clean up the ML models and release the resources
    global model, processor
    model = None
    processor = None

# Initialize FastAPI app - moved after lifespan definition
app = FastAPI(title="CLIP Model Service", lifespan=lifespan)

class HealthResponse(BaseModel):
    status: str

class VectorResponse(BaseModel):
    vector: list[float]
    
@app.get("/api/v1/clip/health", response_model=HealthResponse)
async def health_check():
    return HealthResponse(status="healthy")

@app.post("/api/v1/clip/image-to-vector", response_model=VectorResponse)
async def embed_image(image_base64: str = Body(..., embed=True)):
    try:
        # Decode base64 image
        image_bytes = base64.b64decode(image_base64)
        image = Image.open(io.BytesIO(image_bytes))
        
        # Preprocess image
        inputs = processor(images=image, return_tensors="pt", padding=True)
        
        # Generate embeddings
        with torch.no_grad():
            outputs = model.get_image_features(**inputs)
        
        # Normalize embeddings
        image_embeddings = outputs / outputs.norm(dim=-1, keepdim=True)
        
        # Convert to list for JSON response
        vector = image_embeddings[0].numpy().tolist()
        
        return VectorResponse(vector=vector)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/v1/clip/text-to-vector", response_model=VectorResponse)
async def embed_text(text: str = Body(..., embed=True)):
    try:
        # Preprocess text
        inputs = processor(text=text, return_tensors="pt", padding=True)
        
        # Generate embeddings
        with torch.no_grad():
            outputs = model.get_text_features(**inputs)
        
        # Normalize embeddings
        text_embeddings = outputs / outputs.norm(dim=-1, keepdim=True)
        
        # Convert to list for JSON response
        vector = text_embeddings[0].numpy().tolist()
        
        return VectorResponse(vector=vector)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True) 