package main

import (
	"os"
	"os/signal"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	grpcserver "github.com/reearth/reearth-flow/api/internal/infrastructure/grpc"
	"github.com/reearth/reearthx/log"
)

func main() {
	conf, cerr := config.ReadConfig(false)
	if cerr != nil {
		log.Fatalf("failed to load config: %v", cerr)
	}
	log.Infof("config: %s", conf.Print())

	grpcServer := grpcserver.NewServer(conf.GRPCPort, conf.JWTProviders())

	go func() {
		log.Infof("Starting gRPC server on port %s", conf.GRPCPort)
		if err := grpcServer.Start(); err != nil {
			log.Errorf("Failed to start gRPC server: %v", err)
			os.Exit(1)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit

	log.Info("Shutting down gRPC server...")
}
