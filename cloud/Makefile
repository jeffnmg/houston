# Houston Cloud MVP — common targets

.PHONY: kind-up demo prove-isolation build-engine build-cp

kind-up:
	bash cloud/scripts/kind-up.sh

demo:
	bash cloud/scripts/demo.sh

prove-isolation:
	bash cloud/scripts/prove-isolation.sh

build-engine:
	docker build -t houston/engine:dev -f always-on/Dockerfile .

build-cp:
	docker build -t houston/control-plane:dev -f cloud/control-plane/Dockerfile cloud/control-plane

cp-run-local:
	cd cloud/control-plane && cargo run
