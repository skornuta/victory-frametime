param([string]$Path = "$env:LOCALAPPDATA\victory\cache\frametime.csv")
$lines = Get-Content -LiteralPath $Path
Write-Output "=== header ==="
$lines | Where-Object { $_ -like '#*' } | ForEach-Object { $_ }
$hdr = $lines | Where-Object { $_ -like 't_ms,*' } | Select-Object -First 1
if (-not $hdr) { Write-Output "no CSV header"; exit 1 }
$rows = $lines | Where-Object { $_ -match '^\d' } | ConvertFrom-Csv -Header ($hdr -split ',')
if ($rows.Count -eq 0) { Write-Output "no rows yet"; exit 0 }
function Avg($rs, $c) { ($rs | ForEach-Object { [double]$_.$c } | Measure-Object -Average).Average }
function Med($rs, $c) {
    $v = $rs | ForEach-Object { [double]$_.$c } | Sort-Object
    if ($v.Count -eq 0) { return 0 }
    $v[[int]($v.Count / 2)]
}
$n = $rows.Count
$tailn = [math]::Max(20, [int]($n * 0.4))
$tail = $rows | Select-Object -Last $tailn
Write-Output ("rows={0}  duration_s={1:N0}  tail_rows={2}" -f $n, ((([double]$rows[-1].t_ms - [double]$rows[0].t_ms) / 1000)), $tailn)
Write-Output "--- overall (mean / median) ---"
foreach ($c in 'fps','ms_per_frame','frame_ms_max','cpu_prepare_ms','gpu_ms','copies_ms','textures_ms','copyback_ms','draws','dropped','heap_ms','texture_refresh_ms') {
    "{0,-18} {1,9:N2} / {2,9:N2}" -f $c, (Avg $rows $c), (Med $rows $c)
}
Write-Output "--- settled tail (mean) ---"
foreach ($c in 'fps','ms_per_frame','frame_ms_max','cpu_prepare_ms','gpu_ms','copies_ms','textures_ms','copyback_ms','draws','dropped','heap_ms') {
    "{0,-18} {1,9:N2}" -f $c, (Avg $tail $c)
}
Write-Output "--- 5 slowest intervals ---"
$rows | Sort-Object { [double]$_.ms_per_frame } -Descending | Select-Object -First 5 | ForEach-Object {
    "frame={0,5} fps={1,5:N2} ms={2,8:N1} max={3,7:N1} heap={4,7:N1} tex={5,6:N1} copies={6,6:N1} gpu={7,6:N1} back={8,5:N1} dropped={9,6:N0}" -f `
        $_.frame, [double]$_.fps, [double]$_.ms_per_frame, [double]$_.frame_ms_max, [double]$_.heap_ms, [double]$_.textures_ms, [double]$_.copies_ms, [double]$_.gpu_ms, [double]$_.copyback_ms, [double]$_.dropped
}
