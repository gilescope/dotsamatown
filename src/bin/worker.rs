use dotsamaworld::IOWorker;
use gloo_worker::Registrable;

fn main() {
	console_error_panic_hook::set_once();

	IOWorker::registrar().register();
}
