.PHONY: build clean list

EMPTY_TARGETS = .setup_repo .clone_zenoh_repo

all: sync_zpt_binary

sync_zpt_binary: build_arm
# For Zenoh-2
	rsync -aPz -e "ssh -p 44001" ../../zenoh_performance_tests pi@192.168.1.102:~/
#	rsync -aPz -e "ssh -p 44002" ../../zenoh_performance_tests pi@192.168.1.102:~/
#	rsync -aPz -e "ssh -p 44003" ../../zenoh_performance_tests pi@192.168.1.102:~/
#	rsync -aPz -e "ssh -p 44004" ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44005" ../../zenoh_performance_tests pi@192.168.1.102:~/
#	rsync -aPz -e "ssh -p 44006" ../../zenoh_performance_tests pi@192.168.1.102:~/
# For Zenoh-3
	rsync -aPz -e "ssh -p 44001" ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44002" ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44003" ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44004" ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44005" ../../zenoh_performance_tests pi@192.168.1.103:~/
#	rsync -aPz -e "ssh -p 44006" ../../zenoh_performance_tests pi@192.168.1.103:~/

clean_zpt_binary:
# For Zenoh-2
	ssh -p 44001 pi@192.168.1.102 rm -r zenoh_performance_tests
#	ssh -p 44002 pi@192.168.1.102 rm -r zenoh_performance_tests
#	ssh -p 44003 pi@192.168.1.102 rm -r zenoh_performance_tests
#	ssh -p 44004 pi@192.168.1.102 rm -r zenoh_performance_tests
	ssh -p 44005 pi@192.168.1.102 rm -r zenoh_performance_tests
#	ssh -p 44006 pi@192.168.1.102 rm -r zenoh_performance_tests
# For Zenoh-3
	ssh -p 44001 pi@192.168.1.103 rm -r zenoh_performance_tests
	ssh -p 44002 pi@192.168.1.103 rm -r zenoh_performance_tests
	ssh -p 44003 pi@192.168.1.103 rm -r zenoh_performance_tests
	ssh -p 44004 pi@192.168.1.103 rm -r zenoh_performance_tests
	ssh -p 44005 pi@192.168.1.103 rm -r zenoh_performance_tests
#	ssh -p 44006 pi@192.168.1.103 rm -r zenoh_performance_tests




sync_zenohd_binary: build_zenohd_arm
	rsync -aPz ./zenoh/target/armv7-unknown-linux-gnueabihf/release/zenohd pi@192.168.1.121:~/


build_zenohd_arm: .clone_zenoh_repo
	cd zenoh && cargo build --target armv7-unknown-linux-gnueabihf --release --all

build_arm: .setup_repo
	cd ../ && cargo build --target armv7-unknown-linux-gnueabihf --release --all

.setup_repo:
	cd ../ && git checkout thoughput-latency
	touch .setup_repo

.clone_zenoh_repo:
	git clone https://github.com/eclipse-zenoh/zenoh.git
	cd zenoh && git checkout 90539129b1a7c9e8c7d7daaa84138d093f71fedf
	touch .clone_zenoh_repo

clean:
	rm -rf zenoh
	rm -rf $(EMPTY_TARGETS)

list:
	@LC_ALL=C $(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$'