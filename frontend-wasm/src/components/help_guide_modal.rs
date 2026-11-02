use leptos::*;

#[component]
pub fn HelpGuideModal(
    show_modal: RwSignal<bool>,
) -> impl IntoView {
    let search_query = create_rw_signal("".to_string());
    let active_category = create_rw_signal("todos".to_string());

    view! {
        {move || show_modal.get().then(|| {
            view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-6 bg-slate-900/50 backdrop-blur-xs animate-in fade-in duration-200">
                    // Google Material 3 Help Center Surface Container
                    <div class="bg-white rounded-3xl shadow-[0_24px_50px_rgba(32,33,36,0.22),0_4px_16px_rgba(32,33,36,0.08)] border border-slate-200/90 w-full max-w-3xl max-h-[92vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-150">
                        
                        // Google Iconic 4-Color Accent Strip
                        <div class="h-1 w-full bg-gradient-to-r from-blue-500 via-red-500 via-amber-400 to-emerald-500 shrink-0"></div>

                        // Google Help Center Header
                        <div class="px-6 pt-5 pb-4 bg-white border-b border-slate-100 flex flex-col gap-3 shrink-0">
                            <div class="flex items-start justify-between gap-4">
                                <div class="flex items-center space-x-3">
                                    <div class="w-10 h-10 rounded-full bg-blue-50 border border-blue-100 text-blue-600 flex items-center justify-center text-lg shrink-0 shadow-2xs">
                                        <i class="fa-solid fa-circle-question"></i>
                                    </div>
                                    <div>
                                        <div class="flex items-center space-x-2">
                                            <h2 class="text-base sm:text-lg font-semibold text-slate-900 tracking-tight">
                                                "Centro de Ayuda EcoAlerta"
                                            </h2>
                                            <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-blue-50 text-blue-700 border border-blue-200/60 hidden sm:inline-block">
                                                "Oficial"
                                            </span>
                                        </div>
                                        <p class="text-xs text-slate-500">
                                            "Guía oficial de vigilancia y fiscalización ambiental en Cerro de Pasco"
                                        </p>
                                    </div>
                                </div>
                                <button
                                    class="w-8 h-8 rounded-full text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition flex items-center justify-center cursor-pointer shrink-0"
                                    on:click=move |_| show_modal.set(false)
                                    title="Cerrar ventana de ayuda"
                                >
                                    <i class="fa-solid fa-xmark text-base"></i>
                                </button>
                            </div>

                            // Google Search Input Bar
                            <div class="relative flex items-center w-full mt-1">
                                <i class="fa-solid fa-magnifying-glass text-slate-400 text-xs absolute left-3.5 pointer-events-none"></i>
                                <input
                                    type="text"
                                    placeholder="Buscar temas de ayuda (ej. reportar, SLA, respaldos, botaderos, actas)..."
                                    class="w-full bg-slate-50 hover:bg-slate-100/70 focus:bg-white text-xs text-slate-800 placeholder:text-slate-400 rounded-full py-2.5 pl-9 pr-9 border border-slate-200/80 shadow-2xs focus:border-blue-500 focus:ring-2 focus:ring-blue-100 focus:outline-none transition-all"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                />
                                {move || (!search_query.get().is_empty()).then(|| view! {
                                    <button
                                        class="absolute right-3 text-slate-400 hover:text-slate-600 p-1 rounded-full hover:bg-slate-200 transition cursor-pointer"
                                        title="Limpiar búsqueda"
                                        on:click=move |_| search_query.set("".to_string())
                                    >
                                        <i class="fa-solid fa-xmark text-xs"></i>
                                    </button>
                                })}
                            </div>

                            // Google Material Filter Chips
                            <div class="flex items-center gap-1.5 overflow-x-auto no-scrollbar pt-1">
                                <button
                                    class=move || {
                                        if active_category.get() == "todos" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("todos".to_string())
                                >
                                    "Todo"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "reportar" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("reportar".to_string())
                                >
                                    "📝 Reportes"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "respaldos" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("respaldos".to_string())
                                >
                                    "🤝 Respaldos"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "sla" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("sla".to_string())
                                >
                                    "⏱️ Tiempos SLA"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "acopio" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("acopio".to_string())
                                >
                                    "♻️ Puntos Limpios"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "transparencia" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("transparencia".to_string())
                                >
                                    "📊 Transparencia"
                                </button>
                                <button
                                    class=move || {
                                        if active_category.get() == "autoridades" {
                                            "px-3 py-1 rounded-full text-xs font-semibold bg-blue-600 text-white shadow-2xs transition shrink-0 cursor-pointer"
                                        } else {
                                            "px-3 py-1 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 bg-slate-100 hover:bg-slate-200/80 transition shrink-0 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| active_category.set("autoridades".to_string())
                                >
                                    "🏛️ Autoridades"
                                </button>
                            </div>
                        </div>

                        // Knowledge Base Articles List
                        <div class="flex-1 overflow-y-auto p-4 sm:p-6 space-y-4 text-xs text-slate-700 leading-relaxed bg-slate-50/50">
                            
                            // Article 1: Reportes Ambientales
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "reportar";
                                let match_search = q.is_empty() || "reportar emitir denuncia foto coordenadas pasco".contains(&q) || "¿cómo emitir un reporte ambiental?".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-blue-50 text-blue-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-location-dot"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "¿Cómo emitir un reporte ambiental?"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "Reportes"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "Cualquier ciudadano puede reportar focos de contaminación, relaves sin control, acumulación de basura o vertidos que amenacen la salud pública en los distritos de Chaupimarca, Yanacancha y Simón Bolívar."
                                        </p>

                                        <div class="space-y-2 pt-1 border-t border-slate-100 text-xs text-slate-600">
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"1"</span>
                                                <span><strong>"Ubica el foco en el mapa:"</strong> " Pulsa el botón " <span class="px-1.5 py-0.5 bg-emerald-50 text-emerald-700 rounded font-semibold border border-emerald-200">"'+ Reportar'"</span> " en el panel y haz clic en la ubicación exacta del suceso."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"2"</span>
                                                <span><strong>"Elige la categoría:"</strong> " Selecciona si se trata de Minería/Relaves, Residuos Sólidos, Contaminación de Agua o Emisiones de Aire."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"3"</span>
                                                <span><strong>"Evidencia fotográfica:"</strong> " Adjunta hasta 2 fotografías reales tomadas en el lugar para que las autoridades verifiquen la gravedad."</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-blue-50/70 border border-blue-100/80 rounded-xl p-3 text-[11px] text-blue-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-lightbulb text-blue-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Sugerencia de Google EcoAlerta:"</strong> " El sistema geolocaliza automáticamente la cota altitudinal y coordenadas UTM de Cerro de Pasco para respaldar actas con validez legal."</span>
                                        </div>
                                    </div>
                                })
                            }}

                            // Article 2: Respaldos Vecinales
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "respaldos";
                                let match_search = q.is_empty() || "respaldo vecinos comunidad duplicidad firma prioridad".contains(&q) || "validación y respaldo comunitario".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-emerald-50 text-emerald-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-users"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "Validación y Respaldo Comunitario"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "Comunidad"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "Para prevenir la saturación con reportes duplicados a menos de 50 metros del mismo foco ambiental, EcoAlerta implementa el sistema de respaldo vecinal colaborativo."
                                        </p>

                                        <div class="space-y-2 pt-1 border-t border-slate-100 text-xs text-slate-600">
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"1"</span>
                                                <span><strong>"Consulta el mapa antes de reportar:"</strong> " Si un vecino ya alertó sobre el mismo problema, haz clic sobre el marcador existente en lugar de crear un duplicado."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"2"</span>
                                                <span><strong>"Pulsa 'Respaldar Denuncia':"</strong> " Tu confirmación suma peso y veracidad inmediata a la alerta comunitaria."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"3"</span>
                                                <span><strong>"Elevación automática de prioridad:"</strong> " Las denuncias con 3 o más respaldos suben al inicio de la lista de fiscalización municipal."</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-emerald-50/70 border border-emerald-100/80 rounded-xl p-3 text-[11px] text-emerald-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-shield-halved text-emerald-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Impacto vecinal:"</strong> " Un caso con múltiples respaldos es considerado emergencia ambiental por las autoridades distritales."</span>
                                        </div>
                                    </div>
                                })
                            }}

                            // Article 3: Acuerdos SLA
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "sla";
                                let match_search = q.is_empty() || "sla tiempos atencion critico medio horas oefa fiscalia".contains(&q) || "acuerdos de nivel de servicio".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-amber-50 text-amber-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-clock"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "Tiempos de Atención y Compromisos de Nivel de Servicio (SLA)"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "SLA Municipal"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "Cada denuncia categorizada activa un cronómetro de fiscalización obligatoria con plazos máximos normados según el grado de severidad ambiental:"
                                        </p>

                                        // SLA Cards Grid (Google Material style)
                                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-1">
                                            <div class="p-3 rounded-xl bg-red-50/60 border border-red-100 flex flex-col justify-between">
                                                <div>
                                                    <div class="flex items-center justify-between mb-1">
                                                        <span class="text-[10px] font-bold text-red-800 uppercase tracking-wider">"Crítico"</span>
                                                        <span class="text-xs font-bold font-mono text-red-700 bg-red-100 px-1.5 py-0.5 rounded">"&lt; 24h"</span>
                                                    </div>
                                                    <p class="text-[11px] text-slate-700 leading-snug">
                                                        "Derrames de relaves, quema tóxica o contaminación en lagunas y ríos."
                                                    </p>
                                                </div>
                                                <span class="text-[9px] font-semibold text-red-700 mt-2 block">"⚠️ Notificación a FEMA/OEFA"</span>
                                            </div>

                                            <div class="p-3 rounded-xl bg-amber-50/60 border border-amber-100 flex flex-col justify-between">
                                                <div>
                                                    <div class="flex items-center justify-between mb-1">
                                                        <span class="text-[10px] font-bold text-amber-800 uppercase tracking-wider">"Medio"</span>
                                                        <span class="text-xs font-bold font-mono text-amber-700 bg-amber-100 px-1.5 py-0.5 rounded">"&lt; 48h"</span>
                                                    </div>
                                                    <p class="text-[11px] text-slate-700 leading-snug">
                                                        "Botaderos clandestinos, acumulación excesiva de residuos y desmontes."
                                                    </p>
                                                </div>
                                                <span class="text-[9px] font-semibold text-amber-700 mt-2 block">"🚜 Cuadrilla de limpieza"</span>
                                            </div>

                                            <div class="p-3 rounded-xl bg-emerald-50/60 border border-emerald-100 flex flex-col justify-between">
                                                <div>
                                                    <div class="flex items-center justify-between mb-1">
                                                        <span class="text-[10px] font-bold text-emerald-800 uppercase tracking-wider">"Bajo"</span>
                                                        <span class="text-xs font-bold font-mono text-emerald-700 bg-emerald-100 px-1.5 py-0.5 rounded">"&lt; 72h"</span>
                                                    </div>
                                                    <p class="text-[11px] text-slate-700 leading-snug">
                                                        "Mantenimiento de áreas verdes, limpieza preventiva y desmalezado urbano."
                                                    </p>
                                                </div>
                                                <span class="text-[9px] font-semibold text-emerald-700 mt-2 block">"📅 Programación regular"</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-amber-50/70 border border-amber-100/80 rounded-xl p-3 text-[11px] text-amber-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-bell text-amber-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Garantía de Transparencia:"</strong> " Si un caso excede su plazo sin atención, el estado cambia a 'Vencido' de forma pública e irrevocable en el portal ciudadano."</span>
                                        </div>
                                    </div>
                                })
                            }}

                            // Article 4: Puntos Limpios & Camión
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "acopio";
                                let match_search = q.is_empty() || "acopio botaderos camion ruta reciclaje residuos limpio basuras".contains(&q) || "puntos de acopio y camión recolector".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-teal-50 text-teal-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-recycle"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "Puntos de Acopio, Botaderos y Ruta del Camión"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "Rutas & Residuos"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "Para mantener limpia nuestra ciudad, puedes consultar los puntos oficiales de disposición de residuos y los horarios de cobertura de las unidades compactadoras."
                                        </p>

                                        <div class="space-y-2 pt-1 border-t border-slate-100 text-xs text-slate-600">
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"1"</span>
                                                <span><strong>"Capa '♻️ Acopio & Rutas':"</strong> " Haz clic en el botón superior del mapa para visualizar los contenedores municipales autorizados y la ruta activa del camión."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"2"</span>
                                                <span><strong>"Botón 'Puntos Cercanos':"</strong> " Ordena automáticamente todos los puntos de acopio por distancia exacta desde tu ubicación GPS."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"3"</span>
                                                <span><strong>"Evita multas:"</strong> " Depositar residuos en botaderos informales conlleva sanciones de hasta 1 UIT impuestas por las municipalidades distritales."</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-teal-50/70 border border-teal-100/80 rounded-xl p-3 text-[11px] text-teal-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-truck text-teal-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Horarios de recojo:"</strong> " Los camiones recorren Chaupimarca y Yanacancha en dos turnos: de 06:00 a 09:00 y de 18:00 a 21:00 de lunes a sábado."</span>
                                        </div>
                                    </div>
                                })
                            }}

                            // Article 5: Portal de Transparencia
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "transparencia";
                                let match_search = q.is_empty() || "transparencia actas antes despues csv exportar pdf fiscalizacion balances".contains(&q) || "portal de transparencia ciudadana".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-purple-50 text-purple-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-chart-pie"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "Portal de Transparencia y Actas de Fiscalización"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "Auditoría"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "El portal de transparencia permite a cualquier ciudadano u organización comunitaria auditar el cumplimiento real de las remediaciones ejecutadas por los municipios."
                                        </p>

                                        <div class="space-y-2 pt-1 border-t border-slate-100 text-xs text-slate-600">
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"1"</span>
                                                <span><strong>"Comparativa Visual 'Antes' y 'Después':"</strong> " Cada caso resuelto cuenta con fotografía de la infracción inicial y de la zona remediada por las cuadrillas."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"2"</span>
                                                <span><strong>"Exportación de Balances en CSV:"</strong> " Descarga tablas oficiales estructuradas de casos para informes barriales y asambleas comunales."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"3"</span>
                                                <span><strong>"Acta Técnica Oficial en PDF:"</strong> " Genera e imprime actas con sello de fiscalización, metadatos y coordenadas listas para trámites administrativos."</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-purple-50/70 border border-purple-100/80 rounded-xl p-3 text-[11px] text-purple-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-file-pdf text-purple-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Validez administrativa:"</strong> " El botón 'Imprimir Acta Oficial' dentro de cada caso resuelto formatea automáticamente el expediente para archivado físico."</span>
                                        </div>
                                    </div>
                                })
                            }}

                            // Article 6: Control de Autoridades
                            {move || {
                                let q = search_query.get().to_lowercase();
                                let cat = active_category.get();
                                let match_cat = cat == "todos" || cat == "autoridades";
                                let match_search = q.is_empty() || "autoridades fiscalizador inspector estado credenciales seguridad".contains(&q) || "acceso y panel de autoridades".to_lowercase().contains(&q);

                                (match_cat && match_search).then(|| view! {
                                    <div class="bg-white border border-slate-200/80 rounded-2xl p-4 sm:p-5 shadow-[0_1px_3px_rgba(60,64,67,0.06)] hover:shadow-sm transition space-y-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-2.5">
                                                <div class="w-8 h-8 rounded-full bg-indigo-50 text-indigo-600 flex items-center justify-center text-sm font-semibold shrink-0">
                                                    <i class="fa-solid fa-building-columns"></i>
                                                </div>
                                                <h3 class="text-sm font-semibold text-slate-900">
                                                    "Acceso y Operación para Autoridades Fiscalizadoras"
                                                </h3>
                                            </div>
                                            <span class="text-[10px] font-medium text-slate-500 bg-slate-100 px-2.5 py-0.5 rounded-full">
                                                "Institucional"
                                            </span>
                                        </div>

                                        <p class="text-xs text-slate-600 leading-relaxed">
                                            "El panel de autoridades está restringido exclusivamente a funcionarios con credenciales de fiscalizador ambiental debidamente autenticados."
                                        </p>

                                        <div class="space-y-2 pt-1 border-t border-slate-100 text-xs text-slate-600">
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"1"</span>
                                                <span><strong>"Seguridad basada en roles:"</strong> " Los usuarios ciudadanos o no autenticados tienen acceso de sólo lectura para salvaguardar la integridad de los expedientes."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"2"</span>
                                                <span><strong>"Gestión de Estado de Intervención:"</strong> " Permite transicionar casos de 'Pendiente' a 'En Proceso' y finalmente a 'Resuelto'."</span>
                                            </div>
                                            <div class="flex items-start space-x-2.5">
                                                <span class="w-5 h-5 rounded-full bg-slate-100 text-slate-700 text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">"3"</span>
                                                <span><strong>"Carga de Evidencia Remediada:"</strong> " El cierre formal de una denuncia exige adjuntar la foto certificada del trabajo de limpieza concluido."</span>
                                            </div>
                                        </div>

                                        // Google Support Tip Box
                                        <div class="bg-indigo-50/70 border border-indigo-100/80 rounded-xl p-3 text-[11px] text-indigo-900 flex items-start space-x-2.5">
                                            <i class="fa-solid fa-lock text-indigo-600 mt-0.5 shrink-0 text-xs"></i>
                                            <span><strong>"Control de acceso:"</strong> " Para solicitar credenciales de fiscalizador distrital, diríjase a la Gerencia de Gestión Ambiental de Pasco."</span>
                                        </div>
                                    </div>
                                })
                            }}

                        </div>

                        // Google Material 3 Modal Footer
                        <div class="px-6 py-3.5 bg-white border-t border-slate-100 flex flex-col sm:flex-row items-center justify-between gap-3 shrink-0">
                            <div class="flex items-center space-x-2 text-xs text-slate-500">
                                <i class="fa-solid fa-headset text-slate-400"></i>
                                <span>"¿Dudas adicionales? Mesa de Partes: "</span>
                                <span class="font-medium text-slate-700">"ambiente@munipasco.gob.pe"</span>
                            </div>

                            <div class="flex items-center space-x-2 w-full sm:w-auto justify-end">
                                <button
                                    class="px-4 py-2 rounded-full text-xs font-medium text-slate-600 hover:text-slate-900 hover:bg-slate-100 transition cursor-pointer"
                                    on:click=move |_| show_modal.set(false)
                                >
                                    "Cerrar"
                                </button>
                                <button
                                    class="px-5 py-2 rounded-full text-xs font-semibold text-white bg-blue-600 hover:bg-blue-700 active:bg-blue-800 shadow-sm hover:shadow transition flex items-center space-x-1.5 cursor-pointer"
                                    on:click=move |_| show_modal.set(false)
                                >
                                    <span>"Comenzar a explorar"</span>
                                    <i class="fa-solid fa-arrow-right text-[10px]"></i>
                                </button>
                            </div>
                        </div>

                    </div>
                </div>
            }
        })}
    }
}
