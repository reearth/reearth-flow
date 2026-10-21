// The API schema: system-of-record tables, applied once per environment.
env "local" {
  src = "file://schema.hcl"
  dev = "docker://postgres/18/dev?search_path=public"
  migration {
    dir = "file://migrations"
  }
  format {
    migrate {
      diff = "{{ sql . \"  \" }}"
    }
  }
}

// Applies pending API migrations to DATABASE_URL; the migrations image (Dockerfile) runs this on deploy.
env "deploy" {
  url = getenv("DATABASE_URL")
  migration {
    dir = "file://migrations"
  }
}

// The websocket-go coordination schema: unlogged scratch tables, its own database.
// Kept separate so neither service's migrations run against the other's database.
env "ws" {
  src = "file://ws-schema.hcl"
  dev = "docker://postgres/18/dev?search_path=public"
  migration {
    dir = "file://ws-migrations"
  }
  format {
    migrate {
      diff = "{{ sql . \"  \" }}"
    }
  }
}
