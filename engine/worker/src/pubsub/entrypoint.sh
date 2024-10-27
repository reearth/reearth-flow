#!/bin/bash

set -em

export PUBSUB_PROJECT_ID=$PROJECT_ID
export PUBSUB_EMULATOR_HOST=0.0.0.0:8085

gcloud beta emulators pubsub start --project=$PUBSUB_PROJECT_ID --host-port=$PUBSUB_EMULATOR_HOST --quiet &

while ! nc -z localhost 8085; do
  sleep 0.1
done

. env/bin/activate
# Create topics and subscriptions
IFS=',' read -ra TOPICS <<< "$TOPIC_IDS"
for TOPIC in "${TOPICS[@]}"; do
    # Create a topic
    python3 publisher.py $PUBSUB_PROJECT_ID create $TOPIC

    # Create a pull type subscription
    SUBSCRIPTION_ID="$TOPIC-sub"
    python3 subscriber.py $PUBSUB_PROJECT_ID create $TOPIC $SUBSCRIPTION_ID

    # Create a push type subscription
    # python3 subscriber.py $PUBSUB_PROJECT_ID create-push $TOPIC $SUBSCRIPTION_ID $PUSH_ENDPOINT
done

fg %1