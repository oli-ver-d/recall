## build

~~~zsh
docker build -t recall .
~~~

## run

~~~zsh
docker run -p 8000:8000 -v "$(pwd)/saved_pages:/app/saved_pages" -v "$(pwd)/data:/app/data" recall
~~~

## br

~~~zsh
docker build -t recall .
docker run -i -p 8000:8000 -v "$(pwd)/saved_pages:/app/saved_pages" -v "$(pwd)/data:/app/data" recall
~~~