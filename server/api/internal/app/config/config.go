package config

import (
	"net/url"
	"os"
	"strings"

	"github.com/joho/godotenv"
	"github.com/k0kubun/pp/v3"
	"github.com/kelseyhightower/envconfig"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/mailer"
	"github.com/samber/lo"
)

const configPrefix = "REEARTH_FLOW"

func init() {
	pp.Default.SetColoringEnabled(false)
}

type (
	Mailer mailer.Mailer
	Config struct {
		mailer.Config
		AccountsApiHost     string            `envconfig:"REEARTH_ACCOUNTS_API_HOST" pp:",omitempty"`
		AssetBaseURL        string            `default:"http://localhost:8080/assets"`
		DB                  string            `default:"mongodb://localhost"`
		DB_Account          string            `pp:",omitempty"`
		DB_Users            []appx.NamedURI   `pp:",omitempty"`
		Dev                 bool              `pp:",omitempty"`
		GCPProject          string            `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
		GCPRegion           string            `envconfig:"GOOGLE_CLOUD_REGION" pp:",omitempty"`
		GraphQL             GraphQLConfig     `pp:",omitempty"`
		Host                string            `default:"http://localhost:8080"`
		Host_Web            string            `pp:",omitempty"`
		HTTPSREDIRECT       bool              `pp:",omitempty"`
		Origins             []string          `pp:",omitempty"`
		Port                string            `default:"8080"`
		Profiler            string            `pp:",omitempty"`
		ServerHost          string            `pp:",omitempty"`
		SharedPath          string            `default:"shared"`
		SignupDisabled      bool              `pp:",omitempty"`
		SignupSecret        string            `pp:",omitempty"`
		SkipPermissionCheck bool              `default:"false"`
		Tracer              string            `pp:",omitempty"`
		TracerSample        float64           `pp:",omitempty"`
		Web                 map[string]string `pp:",omitempty"`
		Web_App_Disabled    bool              `pp:",omitempty"`
		Web_Config          JSON              `pp:",omitempty"`
		Web_Disabled        bool              `pp:",omitempty"`
		Web_FaviconURL      string            `pp:",omitempty"`
		Web_Title           string            `pp:",omitempty"`
		WorkflowBaseURL     string            `default:"http://localhost:8080/workflows"`

		// storage
		GCS GCSConfig `pp:",omitempty"`
		S3  S3Config  `pp:",omitempty"`

		// log
		Redis_URL string `pp:",omitempty"`

		// auth
		Auth          AuthConfigs   `pp:",omitempty"`
		Auth0         Auth0Config   `pp:",omitempty"`
		Cognito       CognitoConfig `pp:",omitempty"`
		AuthSrv       AuthSrvConfig `pp:",omitempty"`
		Auth_ISS      string        `pp:",omitempty"`
		Auth_AUD      string        `pp:",omitempty"`
		Auth_ALG      *string       `pp:",omitempty"`
		Auth_TTL      *int          `pp:",omitempty"`
		Auth_ClientID *string       `pp:",omitempty"`
		Auth_JWKSURI  *string       `pp:",omitempty"`

		// worker
		Worker_AllowedLocations                []string `envconfig:"WORKER_BATCH_ALLOWED_LOCATIONS" pp:",omitempty"`
		Worker_BatchSAEmail                    string   `envconfig:"WORKER_BATCH_SA_EMAIL" pp:",omitempty"`
		Worker_BinaryPath                      string   `envconfig:"WORKER_BINARY_PATH" default:"reearth-flow-worker" pp:",omitempty"`
		Worker_BootDiskSizeGB                  string   `envconfig:"WORKER_BOOT_DISK_SIZE_GB" default:"50" pp:",omitempty"`
		Worker_BootDiskType                    string   `envconfig:"WORKER_BOOT_DISK_TYPE" default:"pd-balanced" pp:",omitempty"`
		Worker_ComputeCpuMilli                 string   `envconfig:"WORKER_COMPUTE_CPU_MILLI" default:"2000" pp:",omitempty"`
		Worker_ComputeMemoryMib                string   `envconfig:"WORKER_COMPUTE_MEMORY_MIB" default:"2000" pp:",omitempty"`
		Worker_ImageURL                        string   `envconfig:"WORKER_IMAGE_URL" pp:",omitempty"`
		Worker_MachineType                     string   `envconfig:"WORKER_MACHINE_TYPE" default:"e2-standard-4" pp:",omitempty"`
		Worker_MaxConcurrency                  string   `envconfig:"WORKER_MAX_CONCURRENCY" default:"4" pp:",omitempty"`
		Worker_NodeStatusPropagationDelayMS    string   `envconfig:"WORKER_NODE_STATUS_PROPAGATION_DELAY_MS" default:"1000" pp:",omitempty"`
		Worker_PubSubEdgePassThroughEventTopic string   `envconfig:"WORKER_PUBSUB_EDGE_PASS_THROUGH_EVENT_TOPIC" default:"flow-edge-pass-through" pp:",omitempty"`
		Worker_PubSubJobCompleteTopic          string   `envconfig:"WORKER_PUBSUB_JOB_COMPLETE_TOPIC" default:"flow-job-complete" pp:",omitempty"`
		Worker_PubSubLogStreamTopic            string   `envconfig:"WORKER_PUBSUB_LOG_STREAM_TOPIC" default:"flow-log-stream" pp:",omitempty"`
		Worker_PubSubNodeStatusTopic           string   `envconfig:"WORKER_PUBSUB_NODE_STATUS_TOPIC" default:"flow-node-status" pp:",omitempty"`
		Worker_PubSubStdoutLogTopic            string   `envconfig:"WORKER_PUBSUB_STDOUT_LOG_TOPIC" default:"flow-worker-stdout-log" pp:",omitempty"`
		Worker_TaskCount                       string   `envconfig:"WORKER_TASK_COUNT" default:"1" pp:",omitempty"`

		// websocket
		WebsocketThriftServerURL string `envconfig:"REEARTH_FLOW_WEBSOCKET_THRIFT_SERVER_URL" default:"http://localhost:8000" pp:",omitempty"`
	}
)

func ReadConfig(debug bool) (*Config, error) {
	// load .env
	if err := godotenv.Load(".env"); err != nil && !os.IsNotExist(err) {
		return nil, err
	} else if err == nil {
		log.Infof("config: .env loaded")
	}

	var c Config
	err := envconfig.Process(configPrefix, &c)

	// default values
	if debug {
		c.Dev = true
	}

	c.Host = addHTTPScheme(c.Host)
	if c.Host_Web == "" {
		c.Host_Web = c.Host
	} else {
		c.Host_Web = addHTTPScheme(c.Host_Web)
	}

	if c.AuthSrv.Domain == "" {
		c.AuthSrv.Domain = c.Host
	} else {
		c.AuthSrv.Domain = addHTTPScheme(c.AuthSrv.Domain)
	}

	if c.Host_Web == "" {
		c.Host_Web = c.Host
	}

	if c.AuthSrv.UIDomain == "" {
		c.AuthSrv.UIDomain = c.Host_Web
	} else {
		c.AuthSrv.UIDomain = addHTTPScheme(c.AuthSrv.UIDomain)
	}

	return &c, err
}

func (c *Config) Print() string {
	s := pp.Sprint(c)
	for _, secret := range c.secrets() {
		if secret == "" {
			continue
		}
		s = strings.ReplaceAll(s, secret, "***")
	}
	return s
}

func (c *Config) secrets() []string {
	s := []string{c.DB, c.Auth0.ClientSecret}
	for _, ac := range c.DB_Users {
		s = append(s, ac.URI)
	}
	return s
}

func (c *Config) HostURL() *url.URL {
	u, err := url.Parse(c.Host)
	if err != nil {
		u = nil
	}
	return u
}

func (c *Config) HostWebURL() *url.URL {
	u, err := url.Parse(c.Host_Web)
	if err != nil {
		u = nil
	}
	return u
}

func (c *Config) AuthConfigs() []AuthProvider {
	return []AuthProvider{c.Auth0, c.Cognito}
}

func (c *Config) Auths() (res AuthConfigs) {
	res = lo.FlatMap(c.AuthConfigs(), func(c AuthProvider, _ int) []AuthConfig { return c.Configs() })
	if c.Auth_ISS != "" {
		var aud []string
		if len(c.Auth_AUD) > 0 {
			aud = append(aud, c.Auth_AUD)
		}
		res = append(res, AuthConfig{
			ISS:      c.Auth_ISS,
			AUD:      aud,
			ALG:      c.Auth_ALG,
			TTL:      c.Auth_TTL,
			ClientID: c.Auth_ClientID,
			JWKSURI:  c.Auth_JWKSURI,
		})
	}
	if ac := c.AuthSrv.AuthConfig(c.Dev, c.Host); ac != nil {
		res = append(res, *ac)
	}
	return append(res, c.Auth...)
}

func (c *Config) JWTProviders() (res []appx.JWTProvider) {
	return c.Auths().JWTProviders()
}

func (c *Config) AuthForWeb() *AuthConfig {
	if ac := c.Auth0.AuthConfigForWeb(); ac != nil {
		return ac
	}
	if c.Auth_ISS != "" {
		var aud []string
		if len(c.Auth_AUD) > 0 {
			aud = append(aud, c.Auth_AUD)
		}
		return &AuthConfig{
			ISS:      c.Auth_ISS,
			AUD:      aud,
			ALG:      c.Auth_ALG,
			TTL:      c.Auth_TTL,
			ClientID: c.Auth_ClientID,
		}
	}
	if ac := c.AuthSrv.AuthConfig(c.Dev, c.Host); ac != nil {
		return ac
	}
	return nil
}

func (c *Config) WebConfig() map[string]any {
	w := make(map[string]any)
	for k, v := range c.Web {
		w[k] = v
	}
	if m, ok := c.Web_Config.Data.(map[string]any); ok {
		for k, v := range m {
			w[k] = v
		}
	}
	return w
}

func addHTTPScheme(host string) string {
	if host == "" {
		return ""
	}
	if !strings.HasPrefix(host, "https://") && !strings.HasPrefix(host, "http://") {
		host = "http://" + host
	}
	return host
}
