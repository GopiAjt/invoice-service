#!/bin/sh

echo "Waiting for postgres..."

sleep 5

echo "Running migrations..."

sqlx migrate run

echo "Starting application..."

./invoice-service