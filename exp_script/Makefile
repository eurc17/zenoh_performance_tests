.PHONY: build clean list

EMPTY_TARGETS = .setup_repo .clone_zenoh_repo

all: sync_zpt_binary

sync_zpt_binary: build_arm
# For Zenoh-2
	rsync -aPz -e "ssh -p 44001" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44002" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44003" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44004" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44005" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	rsync -aPz -e "ssh -p 44006" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.102:~/
	ssh -p 44001 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44002 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44003 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44004 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44005 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44006 pi@192.168.1.102 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	rsync -aPz -e "ssh -p 44001" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44002" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44003" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44004" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44005" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44006" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.102:~/zenoh_performance_tests/target/release


# For Zenoh-3
	rsync -aPz -e "ssh -p 44001" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44002" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44003" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44004" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44005" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	rsync -aPz -e "ssh -p 44006" --exclude 'cargo_home' --exclude 'target' --exclude 'armv7l-linux-musleabihf-cross' --exclude 'armv7l-linux-musleabihf-cross.tgz' ../../zenoh_performance_tests pi@192.168.1.103:~/
	ssh -p 44001 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44002 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44003 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44004 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44005 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	ssh -p 44006 pi@192.168.1.103 "cd ~/zenoh_performance_tests && mkdir -p target/release"
	rsync -aPz -e "ssh -p 44001" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44002" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44003" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44004" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44005" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release
	rsync -aPz -e "ssh -p 44006" --exclude ../../zenoh_performance_tests/target/armv7-unknown-linux-musleabihf/release/pub-sub-worker pi@192.168.1.103:~/zenoh_performance_tests/target/release

clean_zpt_binary:
# For Zenoh-2
	ssh -p 44001 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
	ssh -p 44002 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
	ssh -p 44003 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
	ssh -p 44004 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
	ssh -p 44005 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
	ssh -p 44006 pi@192.168.1.102 "rm -r zenoh_performance_tests || true"
# For Zenoh-3
	ssh -p 44001 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"
	ssh -p 44002 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"
	ssh -p 44003 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"
	ssh -p 44004 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"
	ssh -p 44005 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"
	ssh -p 44006 pi@192.168.1.103 "rm -r zenoh_performance_tests || true"


run_zenohd:
	ssh pi@192.168.1.121 "RUST_LOG=info zenohd" &

stop_zenohd:
	ssh pi@192.168.1.121 "pkill zenohd"


build_arm: .setup_repo
	../build_pub_sub_worker.sh

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
	cd ../ && cargo clean

list:
	@LC_ALL=C $(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$'