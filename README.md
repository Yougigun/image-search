# image-search

## Architecture
#### image process flow
<img width="381" alt="image" src="https://github.com/user-attachments/assets/c0b66104-d035-493e-874d-eb7fcb561f06" />

#### image search flow
<img width="455" alt="image" src="https://github.com/user-attachments/assets/9b462e32-6685-4a17-96f0-46241490dcd9" />

#### feedback record flow
<img width="402" alt="image" src="https://github.com/user-attachments/assets/19a9dad6-3748-45f6-b553-22e3810bfdd1" />

## Technical Design (MVP):

### Functions:
1.	Similar Image Search
2.	Record User Feedback

### Overview:
This is the initial design for an image search engine. To handle the processing of images efficiently, I have chosen to use a worker-based system to asynchronously process any images uploaded to the /images folder. This approach is essential because the image-to-embedding process is computationally intensive and time-consuming, and new images are constantly being uploaded.

In the future, we can enhance scalability by leveraging blob storage solutions such as Amazon S3 or a self-hosted object storage service.

For vector storage and similar image search, I have selected Qdrant, a vector database. This choice ensures the system can manage an ever-growing number of images effectively, providing robust and scalable similarity search functionality.

### Feedback Endpoint:
To record user feedback securely, I utilize JWT tokens for authentication, ensuring that requests are valid and authorized.


## How to Launch the Image Search Service

### Prerequisites
- Make command: Ensure make is installed on your system.
- Docker & Docker Compose: Required for containerized services.

### Steps to Launch the Service
 
```bash
# Build the service:
make build
```
```bash
# Run the service:
make run
```
Note: The startup process might take some time because the model service needs to download the weights for the CLIP model.

```bask
# clean up the local
make down
```

### Uploading Images

To upload images:
1.	Copy the image files to the /images folder located at the root of the project.
2.	The worker will automatically detect any new images in the folder and process them into embeddings.

## API 
### search image example
```bash
curl --location 'http://localhost:3000/api/v1/search-image' \
--header 'Content-Type: application/json' \
--data '{
    "text":"tennis"
}'
```
example response 
```json
{
  "text": "tennis",                      // The query text provided for the image search
  "model_name": "CLIP",                // The model used for processing (in this case, CLIP)
  "matches": [                         // Array of matched images with their respective scores
    {
      "image_name": "COCO_val2014_000000000962.jpg", // Name of the matched image
      "score": 0.28906357             // Similarity score between the query text and the image
    }
  ],
  "jwt": "jwt_token_used_in_feedback"  // JWT token for authenticating the feedback request
}
```
### record the feedback
user's feedback range from 1 to 10.
```bash
curl --location 'http://localhost:3000/api/v1/create-feedback' \
--header 'Content-Type: application/json' \
--data '{
    "user_feedback":5,
    "jwt":"jwt_token_used_in_feedback"
}'
```
