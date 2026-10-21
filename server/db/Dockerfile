FROM arigaio/atlas:1.3.4-community-alpine

WORKDIR /db
COPY atlas.hcl .
COPY migrations migrations

CMD ["migrate", "apply", "--env", "deploy"]
