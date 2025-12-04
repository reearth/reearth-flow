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
		AuthSrv      AuthSrvConfig     `pp:",omitempty"`
		Web_Config   JSON              `pp:",omitempty"`
		Web          map[string]string `pp:",omitempty"`
		AuthAlg      *string           `pp:",omitempty"`
		AuthTtl      *int              `pp:",omitempty"`
		AuthClientid *string           `pp:",omitempty"`
		AuthJwksuri  *string           `pp:",omitempty"`

		mailer.Config
		Auth0   Auth0Config   `pp:",omitempty"`
		Cognito CognitoConfig `pp:",omitempty"`

		// storage
		GCS GCSConfig `pp:",omitempty"`
		S3  S3Config  `pp:",omitempty"`

		AccountsApiHost string `envconfig:"REEARTH_ACCOUNTS_API_HOST" pp:",omitempty"`
		AssetBaseURL    string `default:"http://localhost:8080/assets"`
		DB              string `default:"mongodb://localhost"`
		DbAccount       string `pp:",omitempty"`
		GCPProject      string `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
		GCPRegion       string `envconfig:"GOOGLE_CLOUD_REGION" pp:",omitempty"`
		Host            string `default:"http://localhost:8080"`
		HostWeb         string `pp:",omitempty"`
		Port            string `default:"8080"`
		Profiler        string `pp:",omitempty"`
		ServerHost      string `pp:",omitempty"`
		SharedPath      string `default:"shared"`
		SignupSecret    string `pp:",omitempty"`
		Tracer          string `pp:",omitempty"`
		WebFaviconurl   string `pp:",omitempty"`
		WebTitle        string `pp:",omitempty"`
		WorkflowBaseURL string `default:"http://localhost:8080/workflows"`

		// log
		RedisUrl string `pp:",omitempty"`

		AuthIss                               string `pp:",omitempty"`
		AuthAud                               string `pp:",omitempty"`
		WorkerBatchsaemail                    string `envconfig:"WORKER_BATCHSAEMAIL" pp:",omitempty"`
		WorkerBinarypath                      string `envconfig:"WORKER_BINARYPATH" default:"reearth-flow-worker" pp:",omitempty"`
		WorkerBootdisksizegb                  string `envconfig:"WORKER_BOOTDISKSIZEGB" default:"50" pp:",omitempty"`
		WorkerBootdisktype                    string `envconfig:"WORKER_BOOTDISKTYPE" default:"pd-balanced" pp:",omitempty"`
		WorkerChannelbuffersize               string `envconfig:"WORKER_CHANNELBUFFERSIZE" default:"256" pp:",omitempty"`
		WorkerComputecpumilli                 string `envconfig:"WORKER_COMPUTECPUMILLI" default:"2000" pp:",omitempty"`
		WorkerComputememorymib                string `envconfig:"WORKER_COMPUTEMEMORYMIB" default:"2000" pp:",omitempty"`
		WorkerFeatureflushthreshold           string `envconfig:"WORKER_FEATUREFLUSHTHRESHOLD" default:"512" pp:",omitempty"`
		WorkerImageurl                        string `envconfig:"WORKER_IMAGEURL" pp:",omitempty"`
		WorkerMachinetype                     string `envconfig:"WORKER_MACHINETYPE" default:"e2-standard-4" pp:",omitempty"`
		WorkerMaxconcurrency                  string `envconfig:"WORKER_MAXCONCURRENCY" default:"4" pp:",omitempty"`
		WorkerNodestatuspropagationdelayms    string `envconfig:"WORKER_NODESTATUSPROPAGATIONDELAYMS" default:"1000" pp:",omitempty"`
		WorkerPubsubedgepassthrougheventtopic string `envconfig:"WORKER_PUBSUBEDGEPASSTHROUGHEVENTTOPIC" default:"flow-edge-pass-through" pp:",omitempty"`
		WorkerPubsubjobcompletetopic          string `envconfig:"WORKER_PUBSUBJOBCOMPLETETOPIC" default:"flow-job-complete" pp:",omitempty"`
		WorkerPubsublogstreamtopic            string `envconfig:"WORKER_PUBSUBLOGSTREAMTOPIC" default:"flow-log-stream" pp:",omitempty"`
		WorkerPubsubnodestatustopic           string `envconfig:"WORKER_PUBSUBNODESTATUSTOPIC" default:"flow-node-status" pp:",omitempty"`
		WorkerPubsubuserfacinglogtopic        string `envconfig:"WORKER_PUBSUBUSERFACINGLOGTOPIC" default:"flow-user-facing-log" pp:",omitempty"`
		WorkerTaskcount                       string `envconfig:"WORKER_TASKCOUNT" default:"1" pp:",omitempty"`
		WorkerThreadpoolsize                  string `envconfig:"WORKER_THREADPOOLSIZE" default:"30" pp:",omitempty"`
		WorkerRustlog                         string `envconfig:"WORKER_RUSTLOG" default:"info" pp:",omitempty"`

		// websocket
		WebsocketThriftServerURL string `envconfig:"REEARTH_FLOW_WEBSOCKET_SERVER_URL" default:"http://localhost:8000" pp:",omitempty"`

		// cms
		CmsEndpoint string          `envconfig:"REEARTH_FLOW_GRPC_ENDPOINT_CMS" pp:",omitempty"`
		CmsToken    string          `envconfig:"REEARTH_FLOW_GRPC_TOKEN_CMS" pp:",omitempty"`
		DbUsers     []appx.NamedURI `pp:",omitempty"`
		Origins     []string        `pp:",omitempty"`

		// auth
		Auth AuthConfigs `pp:",omitempty"`

		// worker
		WorkerAllowedlocations         []string      `envconfig:"WORKER_BATCH_ALLOWED_LOCATIONS" pp:",omitempty"`
		GraphQL                        GraphQLConfig `pp:",omitempty"`
		TracerSample                   float64       `pp:",omitempty"`
		AssetUploadURLReplacement      bool          `default:"false" pp:",omitempty"`
		Dev                            bool          `pp:",omitempty"`
		HTTPSREDIRECT                  bool          `pp:",omitempty"`
		SignupDisabled                 bool          `pp:",omitempty"`
		SkipPermissionCheck            bool          `default:"false"`
		WebAppDisabled                 bool          `pp:",omitempty"`
		WebDisabled                    bool          `pp:",omitempty"`
		WorkerCompressintermediatedata bool          `envconfig:"WORKER_COMPRESSINTERMEDIATEDATA" default:"false" pp:",omitempty"`
		CmsUsetls                      bool          `envconfig:"REEARTH_FLOW_GRPC_USETLS" default:"true" pp:",omitempty"`
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
	if c.HostWeb == "" {
		c.HostWeb = c.Host
	} else {
		c.HostWeb = addHTTPScheme(c.HostWeb)
	}

	if c.AuthSrv.Domain == "" {
		c.AuthSrv.Domain = c.Host
	} else {
		c.AuthSrv.Domain = addHTTPScheme(c.AuthSrv.Domain)
	}

	if c.HostWeb == "" {
		c.HostWeb = c.Host
	}

	if c.AuthSrv.UIDomain == "" {
		c.AuthSrv.UIDomain = c.HostWeb
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
	for _, ac := range c.DbUsers {
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
	u, err := url.Parse(c.HostWeb)
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
	if c.AuthIss != "" {
		var aud []string
		if len(c.AuthAud) > 0 {
			aud = append(aud, c.AuthAud)
		}
		res = append(res, AuthConfig{
			ISS:      c.AuthIss,
			AUD:      aud,
			ALG:      c.AuthAlg,
			TTL:      c.AuthTtl,
			ClientID: c.AuthClientid,
			JWKSURI:  c.AuthJwksuri,
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
	if c.AuthIss != "" {
		var aud []string
		if len(c.AuthAud) > 0 {
			aud = append(aud, c.AuthAud)
		}
		return &AuthConfig{
			ISS:      c.AuthIss,
			AUD:      aud,
			ALG:      c.AuthAlg,
			TTL:      c.AuthTtl,
			ClientID: c.AuthClientid,
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
