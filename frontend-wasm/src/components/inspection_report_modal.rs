use leptos::*;
use crate::models::{EcoAlert, EcoCategory, EcoDistrict, EcoSeverity};

#[component]
pub fn InspectionReportModal(
    report_target: RwSignal<Option<EcoAlert>>,
) -> impl IntoView {
    let print_report = move |_| {
        let _ = js_sys::eval("window.print();");
    };

    let close_modal = move |_| {
        report_target.set(None);
    };

    view! {
        {move || report_target.get().map(|alert| {
            let id = alert.id.clone();
            let report_code = if id.len() >= 8 {
                id[..8].to_uppercase()
            } else {
                id.to_uppercase()
            };

            let title = alert.title.clone();
            let reporter = alert.reporter_name.clone();
            let address = alert.address.clone();
            let lat = alert.latitude;
            let lng = alert.longitude;
            let confirmations = alert.confirmations_count;
            let desc = alert.description.clone();
            let created = alert.created_at.clone();
            let resolved_date = alert.resolved_at.clone().unwrap_or_else(|| "Pendiente de dictamen de cierre".to_string());
            let resolution_comment = alert.resolution_comment.clone().unwrap_or_else(|| {
                if alert.status == "solved" {
                    "Trabajos de saneamiento y mitigación ambiental concluidos de conformidad técnica.".to_string()
                } else {
                    "Cuadrilla municipal en proceso de inspección y remediación técnica ambiental.".to_string()
                }
            });

            let district_title = match alert.district {
                EcoDistrict::Yanacancha => "MUNICIPALIDAD DISTRITAL DE YANACANCHA",
                EcoDistrict::SimonBolivar => "MUNICIPALIDAD DISTRITAL DE SIMÓN BOLÍVAR",
                EcoDistrict::Chaupimarca => "MUNICIPALIDAD PROVINCIAL DE PASCO (CHAUPIMARCA)",
            };

            let (cat_icon, cat_label) = match alert.category {
                EcoCategory::Mineria => ("⛏️", "Minería & Pasivos Mineros"),
                EcoCategory::Basura => ("🗑️", "Residuos Sólidos & Botaderos"),
                EcoCategory::Agua => ("💧", "Recursos Hídricos & Efluentes"),
                EcoCategory::Aire => ("🌬️", "Calidad del Aire & Emisiones"),
            };

            let (sev_label, sev_badge_class) = match alert.severity {
                EcoSeverity::Critico => ("CRÍTICO (ALTO IMPACTO)", "bg-red-50 text-red-700 border-red-200"),
                EcoSeverity::Medio => ("MEDIO (IMPACTO MODERADO)", "bg-amber-50 text-amber-700 border-amber-200"),
                EcoSeverity::Bajo => ("BAJO (LEVE)", "bg-emerald-50 text-emerald-700 border-emerald-200"),
            };

            let status_text = match alert.status.as_str() {
                "solved" => "SOLUCIONADO / INTERVENCIÓN CONFORME",
                "dismissed" => "DESCARTADO POR FISCALIZACIÓN",
                _ => "EN PROCESO / CUADRILLA ASIGNADA",
            };

            let img_before = alert.image_url.clone();
            let img_after = alert.resolution_image_url.clone();

            view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-6 bg-slate-900/60 backdrop-blur-sm animate-in fade-in duration-200">
                    <div class="bg-white rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.3)] border border-slate-200 w-full max-w-4xl max-h-[92vh] flex flex-col overflow-hidden">
                        // Header Bar (Hidden during print)
                        <div class="print:hidden px-6 py-3.5 bg-slate-50 border-b border-slate-200 flex items-center justify-between gap-3 shrink-0">
                            <div class="flex items-center space-x-2.5">
                                <div class="w-8 h-8 rounded-xl bg-sky-100 text-sky-700 flex items-center justify-center">
                                    <i class="fa-solid fa-file-contract text-base"></i>
                                </div>
                                <div>
                                    <h3 class="text-sm font-bold text-slate-900">"Vista Previa de Acta Oficial de Fiscalización"</h3>
                                    <p class="text-[11px] text-slate-500 font-mono">"Documento técnico municipal generado por EcoAlerta"</p>
                                </div>
                            </div>
                            <div class="flex items-center space-x-2">
                                <button
                                    class="px-3 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white font-semibold text-xs shadow-xs flex items-center space-x-1.5 transition cursor-pointer"
                                    on:click={
                                        let a_clone = alert.clone();
                                        move |_| {
                                            if let Ok(json_str) = serde_json::to_string(&a_clone) {
                                                let script = format!("window.ecoPdf.generateInspectionActa({});", json_str);
                                                let _ = js_sys::eval(&script);
                                            }
                                        }
                                    }
                                    title="Descarga automática del documento pericial en formato PDF con membrete y QR oficial"
                                >
                                    <i class="fa-solid fa-file-pdf text-xs"></i>
                                    <span>"Descargar PDF"</span>
                                </button>
                                <button
                                    class="px-3.5 py-1.5 rounded-xl bg-sky-600 hover:bg-sky-700 text-white font-semibold text-xs shadow-xs flex items-center space-x-2 transition cursor-pointer"
                                    on:click=print_report
                                    title="Imprimir documento oficial o guardar en PDF"
                                >
                                    <i class="fa-solid fa-print text-xs"></i>
                                    <span>"Imprimir"</span>
                                </button>
                                <button
                                    class="p-1.5 rounded-xl text-slate-400 hover:text-slate-700 hover:bg-slate-200/60 transition cursor-pointer"
                                    on:click=close_modal
                                    title="Cerrar vista previa"
                                >
                                    <i class="fa-solid fa-xmark text-base"></i>
                                </button>
                            </div>
                        </div>

                        // Printable Content (The Formal Peruvian Municipal Acta)
                        <div class="flex-1 overflow-y-auto p-6 sm:p-10 bg-white text-slate-900 space-y-6 print:p-0 print:overflow-visible font-sans">
                            // 1. Municipal Official Header
                            <div class="flex items-start justify-between gap-4 border-b-2 border-slate-900 pb-5">
                                <div class="flex items-center space-x-3.5">
                                    <div class="w-14 h-14 rounded-2xl bg-slate-900 text-white flex items-center justify-center text-2xl shrink-0 shadow-sm">
                                        "🌿"
                                    </div>
                                    <div>
                                        <h1 class="text-base sm:text-lg font-black tracking-wide text-slate-900 uppercase">
                                            {district_title}
                                        </h1>
                                        <h2 class="text-xs font-bold text-slate-600 tracking-wider uppercase">
                                            "Gerencia de Gestión Ambiental, Servicios Públicos y Salud"
                                        </h2>
                                        <p class="text-[10px] text-slate-500">
                                            "Sistema Ciudadano de Alerta Temprana EcoAlerta — Cerro de Pasco, Perú"
                                        </p>
                                    </div>
                                </div>
                                <div class="p-2.5 rounded-xl border border-slate-300 bg-slate-50/70 text-right shrink-0">
                                    <span class="block text-xs font-black text-slate-900 font-mono">
                                        {format!("ACTA N° {}-2026", report_code)}
                                    </span>
                                    <span class="block text-[10px] text-slate-500 font-mono">
                                        "Fecha: Pasco, Perú"
                                    </span>
                                </div>
                            </div>

                            // 2. Title
                            <div class="text-center py-1">
                                <h2 class="text-sm sm:text-base font-black tracking-wide text-slate-900 uppercase underline decoration-2 underline-offset-4">
                                    "ACTA TÉCNICA DE FISCALIZACIÓN E INTERVENCIÓN AMBIENTAL"
                                </h2>
                            </div>

                            // Section I: Datos del Incidente y Ciudadano Reportante
                            <div class="space-y-2">
                                <div class="px-2.5 py-1 bg-slate-100 rounded-md">
                                    <h3 class="text-xs font-extrabold uppercase tracking-wider text-slate-800">
                                        "I. DATOS DEL INCIDENTE Y CIUDADANO DENUNCIANTE"
                                    </h3>
                                </div>
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-1.5 text-xs pt-1 px-1">
                                    <div>
                                        <span class="font-bold text-slate-600">"Título del Caso: "</span>
                                        <span class="font-semibold text-slate-900">{title}</span>
                                    </div>
                                    <div>
                                        <span class="font-bold text-slate-600">"Ciudadano Denunciante: "</span>
                                        <span class="font-semibold text-slate-900">{reporter}</span>
                                    </div>
                                    <div>
                                        <span class="font-bold text-slate-600">"Fecha y Hora de Emisión: "</span>
                                        <span class="font-mono text-slate-800">{created}</span>
                                    </div>
                                    <div>
                                        <span class="font-bold text-slate-600">"Validación Comunitaria: "</span>
                                        <span class="font-semibold text-sky-800">
                                            {format!("👥 {} vecinos respaldaron la veracidad", confirmations)}
                                        </span>
                                    </div>
                                    <div class="sm:col-span-2">
                                        <span class="font-bold text-slate-600">"Ubicación Georreferenciada: "</span>
                                        <span class="text-slate-900">{address}</span>
                                    </div>
                                    <div class="sm:col-span-2">
                                        <span class="font-bold text-slate-600">"Coordenadas GPS Satelitales: "</span>
                                        <span class="font-mono text-slate-800">
                                            {format!("Latitud: {:.6}  |  Longitud: {:.6}", lat, lng)}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            // Section II: Clasificación Técnica y Evaluación
                            <div class="space-y-2">
                                <div class="px-2.5 py-1 bg-slate-100 rounded-md">
                                    <h3 class="text-xs font-extrabold uppercase tracking-wider text-slate-800">
                                        "II. CLASIFICACIÓN TÉCNICA Y EVALUACIÓN DE IMPACTO"
                                    </h3>
                                </div>
                                <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs pt-1 px-1">
                                    <div class="p-2.5 rounded-xl border border-slate-200 bg-slate-50/50">
                                        <span class="block text-[10px] font-bold text-slate-500 uppercase">"Categoría Ambiental"</span>
                                        <span class="font-bold text-slate-900 mt-0.5 block">
                                            {format!("{} {}", cat_icon, cat_label)}
                                        </span>
                                    </div>
                                    <div class="p-2.5 rounded-xl border border-slate-200 bg-slate-50/50">
                                        <span class="block text-[10px] font-bold text-slate-500 uppercase">"Nivel de Severidad"</span>
                                        <span class=format!("inline-block px-2 py-0.5 mt-1 rounded text-[11px] font-bold border {}", sev_badge_class)>
                                            {sev_label}
                                        </span>
                                    </div>
                                    <div class="p-2.5 rounded-xl border border-slate-200 bg-slate-50/50">
                                        <span class="block text-[10px] font-bold text-slate-500 uppercase">"Estado de Gestión"</span>
                                        <span class="font-bold text-emerald-800 mt-0.5 block">
                                            {status_text}
                                        </span>
                                    </div>
                                </div>
                                <div class="px-1 text-xs">
                                    <span class="font-bold text-slate-600">"Descripción del Hecho: "</span>
                                    <p class="mt-1 text-slate-800 leading-relaxed bg-slate-50 p-2.5 rounded-xl border border-slate-200">
                                        {desc}
                                    </p>
                                </div>
                            </div>

                            // Section III: Registro Fotográfico de Peritaje (Antes vs Después)
                            <div class="space-y-2">
                                <div class="px-2.5 py-1 bg-slate-100 rounded-md">
                                    <h3 class="text-xs font-extrabold uppercase tracking-wider text-slate-800">
                                        "III. REGISTRO FOTOGRÁFICO DE PERITAJE (ANTES VS DESPUÉS)"
                                    </h3>
                                </div>
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-1">
                                    // Foto A: Inicial
                                    <div class="space-y-1.5">
                                        <span class="text-[11px] font-bold text-red-700 uppercase">
                                            "A) EVIDENCIA INICIAL (DENUNCIA CIUDADANA)"
                                        </span>
                                        <div class="h-48 w-full rounded-xl overflow-hidden border border-slate-200 bg-slate-100 flex items-center justify-center">
                                            {if let Some(ref url) = img_before {
                                                let u = url.clone();
                                                view! {
                                                    <img src=u alt="Evidencia inicial" class="w-full h-full object-cover" />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="text-center text-slate-400 text-xs">
                                                        <i class="fa-solid fa-image text-2xl block mb-1"></i>
                                                        <span>"Sin fotografía inicial registrada"</span>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                    </div>

                                    // Foto B: Resolución
                                    <div class="space-y-1.5">
                                        <span class="text-[11px] font-bold text-emerald-700 uppercase">
                                            "B) EVIDENCIA DE RESOLUCIÓN (MUNICIPALIDAD)"
                                        </span>
                                        <div class="h-48 w-full rounded-xl overflow-hidden border border-slate-200 bg-slate-100 flex items-center justify-center">
                                            {if let Some(ref url) = img_after {
                                                let u = url.clone();
                                                view! {
                                                    <img src=u alt="Evidencia de resolución" class="w-full h-full object-cover" />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="text-center text-slate-400 text-xs">
                                                        <i class="fa-solid fa-shield-halved text-2xl block mb-1"></i>
                                                        <span>"Caso en proceso de resolución física"</span>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Section IV: Dictamen y Acciones Conclusivas
                            <div class="space-y-2">
                                <div class="px-2.5 py-1 bg-slate-100 rounded-md">
                                    <h3 class="text-xs font-extrabold uppercase tracking-wider text-slate-800">
                                        "IV. ACCIONES REALIZADAS Y CONCLUSIONES OPERATIVAS"
                                    </h3>
                                </div>
                                <div class="px-1 space-y-1.5 text-xs">
                                    <p class="text-slate-800 leading-relaxed bg-slate-50 p-3 rounded-xl border border-slate-200">
                                        <span class="font-bold text-slate-700 block mb-1">"Dictamen Técnico de Intervención:"</span>
                                        {resolution_comment}
                                    </p>
                                    <p class="text-[11px] text-slate-500 font-mono">
                                        <span class="font-bold">"Fecha de Conclusión: "</span>
                                        {resolved_date}
                                    </p>
                                </div>
                            </div>

                            // Section V: Firmas Autorizadas
                            <div class="pt-6 space-y-8">
                                <div class="px-2.5 py-1 bg-slate-100 rounded-md">
                                    <h3 class="text-xs font-extrabold uppercase tracking-wider text-slate-800">
                                        "V. CONFORMIDAD Y FIRMAS AUTORIZADAS"
                                    </h3>
                                </div>
                                <div class="grid grid-cols-2 gap-8 pt-4 pb-2 text-center text-xs">
                                    <div class="space-y-1">
                                        <div class="w-48 mx-auto border-t-2 border-slate-800"></div>
                                        <span class="font-bold text-slate-900 block">"Inspector Técnico Ambiental"</span>
                                        <span class="text-[11px] text-slate-500 block">"Gerencia de Gestión Ambiental"</span>
                                    </div>
                                    <div class="space-y-1">
                                        <div class="w-48 mx-auto border-t-2 border-slate-800"></div>
                                        <span class="font-bold text-slate-900 block">"V°B° Gerente de Servicios Públicos"</span>
                                        <span class="text-[11px] text-slate-500 block">"Municipalidad Competente"</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            }
        })}
    }
}
