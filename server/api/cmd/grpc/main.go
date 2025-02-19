package main

import (
	"context"
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

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	grpcServer := grpcserver.NewServer(conf.GRPCPort, conf.JWTProviders())

	errChan := make(chan error, 1)
	go func() {
		log.Infof("Starting gRPC server on port %s", conf.GRPCPort)
		errChan <- grpcServer.StartWithContext(ctx)
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	select {
	case err := <-errChan:
		if err != nil {
			log.Errorf("Failed to start gRPC server: %v", err)
			os.Exit(1)
		}
	case <-quit:
		log.Info("Shutting down gRPC server...")
		cancel()
	}
}
