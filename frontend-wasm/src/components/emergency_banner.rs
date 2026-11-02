use leptos::*;
use crate::models::EcoAlert;

#[component]
pub fn EmergencyBanner(
    active_emergency: RwSignal<Option<EcoAlert>>,
    inspection_target: RwSignal<Option<EcoAlert>>,
    selected_alert_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let fly_to_emergency = move |a: &EcoAlert| {
        let lat = a.latitude;
        let lng = a.longitude;
        selected_alert_id.set(Some(a.id.clone()));
        let _ = js_sys::eval(&format!("window.ecoMap.flyTo({}, {}, 16.5);", lat, lng));
    };

    view! {
        {move || active_emergency.get().map(|alert| {
            let a = alert.clone();
            let title = alert.title.clone();
            let addr = alert.address.clone();
            let a_for_acta = alert.clone();

            view! {
                <div class="absolute top-3.5 left-1/2 -translate-x-1/2 z-30 max-w-2xl w-11/12 animate-in slide-in-from-top duration-300 pointer-events-auto">
                    <div class="bg-gradient-to-r from-red-600 via-rose-600 to-red-700 text-white rounded-2xl p-3 sm:p-4 shadow-[0_12px_36px_rgba(220,38,38,0.4)] border-2 border-white/80 flex items-center justify-between gap-3">
                        <div class="flex items-center space-x-3 min-w-0">
                            <div class="w-10 h-10 rounded-xl bg-white/20 backdrop-blur-md flex items-center justify-center text-xl shrink-0 animate-bounce">
                                "🚨"
                            </div>
                            <div class="min-w-0">
                                <div class="flex items-center space-x-2">
                                    <span class="px-2 py-0.5 rounded-md bg-white/30 text-[9px] font-black uppercase tracking-wider font-mono">
                                        "Alerta Roja Inmediata"
                                    </span>
                                    <span class="text-[10px] text-white/80 truncate">
                                        "Cerro de Pasco"
                                    </span>
                                </div>
                                <h4 class="text-xs sm:text-sm font-black truncate leading-tight mt-0.5">
                                    {title}
                                </h4>
                                <p class="text-[10px] text-white/90 truncate font-mono">
                                    "📍 " {addr}
                                </p>
                            </div>
                        </div>

                        <div class="flex items-center space-x-2 shrink-0">
                            <button
                                class="px-3 py-1.5 rounded-xl bg-white text-red-700 font-bold text-xs hover:bg-red-50 shadow-xs transition flex items-center space-x-1 cursor-pointer"
                                title="Centrar cámara en el foco de la emergencia"
                                on:click={
                                    let a_clone = a.clone();
                                    move |_| fly_to_emergency(&a_clone)
                                }
                            >
                                <i class="fa-solid fa-location-crosshairs text-xs"></i>
                                <span class="hidden sm:inline">"Ver Foco"</span>
                            </button>

                            <button
                                class="px-3 py-1.5 rounded-xl bg-red-900/60 hover:bg-red-900/80 text-white font-bold text-xs border border-white/30 transition flex items-center space-x-1 cursor-pointer"
                                title="Generar acta técnica oficial"
                                on:click={
                                    let a_acta_clone = a_for_acta.clone();
                                    move |_| inspection_target.set(Some(a_acta_clone.clone()))
                                }
                            >
                                <i class="fa-solid fa-file-contract text-xs"></i>
                                <span class="hidden md:inline">"Acta"</span>
                            </button>

                            <button
                                class="w-8 h-8 rounded-xl bg-black/20 hover:bg-black/40 text-white flex items-center justify-center transition cursor-pointer"
                                title="Descartar banner de emergencia"
                                on:click=move |_| active_emergency.set(None)
                            >
                                <i class="fa-solid fa-xmark text-sm"></i>
                            </button>
                        </div>
                    </div>
                </div>
            }
        })}
    }
}
