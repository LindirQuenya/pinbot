# Start off from an image with python preinstalled.
FROM python:buster

# Create and move to a new directory for our project.
WORKDIR /pinbot
# Copy the needed files over.
COPY token.txt token.txt
COPY runbot.py runbot.py
COPY requirements.txt requirements.txt

# Install our dependencies.
RUN pip install -r requirements.txt

# When run, this is what we execute.
CMD ["python3", "runbot.py"]
