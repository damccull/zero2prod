#!/usr/bin/env bash

set -x
set -eo pipefail

# Cehck if a custom user has been set, or default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, or default to 'password'
DB_PASSWORD="${POSTGRESS_PASSWORD:=password}"
# Check if a custom database name has been set, or default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# Check if a custom port has been set, or default to "5432"
DB_PORT="${POSTGRES_PORT:=5432}"

# Allow skipping the docker stuff if dockerized postgres is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
    CONTAINER_ID=$(docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000)
        # ^ Increase max number of connections for integration testing purposes

    export PGPASSWORD="$DB_PASSWORD"
    timeout 10s bash -c "until docker exec ${CONTAINER_ID} pg_isready ; do sleep 1 ; done"
fi

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@postgres:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run