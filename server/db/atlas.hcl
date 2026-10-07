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
