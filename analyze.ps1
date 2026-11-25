param([string]$Path)
$lines = Get-Content $Path
$hdr = $lines[0] -split ','
$rows = $lines | Select-Object -Skip 1 | Where-Object { $_ -match '^\d' } | ConvertFrom-Csv -Header $hdr
function Avg($rs, $c) {
    $v = $rs | ForEach-Object { [double]$_.$c }
    ($v | Measure-Object -Average).Average
}
function Med($rs, $c) {
    $v = $rs | ForEach-Object { [double]$_.$c } | Sort-Object
    if ($v.Count -eq 0) { return 0 }
    $v[[int]($v.Count / 2)]
}
$n = $rows.Count
$tailn = [math]::Max(20, [int]($n * 0.4))
$tail = $rows | Select-Object -Last $tailn
Write-Output ("SAMPLE " + (Split-Path $Path -Leaf))
Write-Output ("rows={0}  duration_s={1:N0}  tail_rows={2}" -f $n, ((([double]$rows[-1].t_ms - [double]$rows[0].t_ms) / 1000)), $tailn)
Write-Output "--- overall (mean) ---"
foreach ($c in 'fps','ms_per_frame','frame_ms_max','cpu_prepare_ms','gpu_ms','copies_ms','copies_mb','textures_ms','copyback_ms','draws','dropped','heap_ms','texture_refresh_ms','card_wait_ms','ram_mb') {
    "{0,-18} mean {1,9:N2}   median {2,9:N2}" -f $c, (Avg $rows $c), (Med $rows $c)
}
Write-Output "--- settled tail (mean) ---"
foreach ($c in 'fps','ms_per_frame','frame_ms_max','cpu_prepare_ms','gpu_ms','copies_ms','textures_ms','copyback_ms','draws','dropped','heap_ms','texture_refresh_ms') {
    "{0,-18} {1,9:N2}" -f $c, (Avg $tail $c)
}
Write-Output "--- 5 slowest intervals ---"
$rows | Sort-Object { [double]$_.ms_per_frame } -Descending | Select-Object -First 5 | ForEach-Object {
    "frame={0,5} fps={1,5:N2} ms={2,8:N1} max={3,7:N1} heap={4,7:N1} tex={5,6:N1} copies={6,6:N1} gpu={7,6:N1} back={8,5:N1} dropped={9,6:N0}" -f `
        $_.frame, [double]$_.fps, [double]$_.ms_per_frame, [double]$_.frame_ms_max, [double]$_.heap_ms, [double]$_.textures_ms, [double]$_.copies_ms, [double]$_.gpu_ms, [double]$_.copyback_ms, [double]$_.dropped
}
Write-Output "--- 5 slowest single frames (frame_ms_max) ---"
$rows | Sort-Object { [double]$_.frame_ms_max } -Descending | Select-Object -First 5 | ForEach-Object {
    "frame={0,5} max={1,7:N1} avg={2,6:N1} heap={3,7:N1} tex={4,6:N1} copies={5,6:N1} gpu={6,6:N1}" -f `
        $_.frame, [double]$_.frame_ms_max, [double]$_.frame_ms_avg, [double]$_.heap_ms, [double]$_.textures_ms, [double]$_.copies_ms, [double]$_.gpu_ms
}
Write-Output ""
