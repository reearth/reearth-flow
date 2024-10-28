FROM gcr.io/google.com/cloudsdktool/cloud-sdk:427.0.0-emulators

RUN apt-get update && \
    apt-get install -y git python3-pip netcat && \
    git clone --depth 1 --branch v2.26.1 https://github.com/googleapis/python-pubsub.git

WORKDIR /python-pubsub/samples/snippets
RUN pip3 install virtualenv && \
    virtualenv env && \
    . env/bin/activate && \
    pip3 install -r requirements.txt

COPY ./containers/pubsub/entrypoint.sh ./
EXPOSE 8085
ENTRYPOINT ["./entrypoint.sh"]
