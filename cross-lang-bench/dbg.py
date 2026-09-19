import io, re, sys
raw = io.open('/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/BENCHMARKS.md','rb').read()
print('CRLF count:', raw.count(b'\r\n'), 'LF total:', raw.count(b'\n'))
md = raw.decode('utf-8-sig')
print('has Run4:', '# Run 4' in md)
i = md.find('# Run 4')
sec = md[i:]
m = re.search(br'^### single_int: 1 thread', sec.encode('utf-8'), re.M)
print('header match:', m.group(0).decode() if m else None)
for ln in sec.splitlines():
    if 'ticklog' in ln and ln.lstrip().startswith('|'):
        cells = [c.strip() for c in ln.strip().strip('|').split('|')]
        if cells[0] == 'ticklog':
            print('row:', cells, 'len', len(cells))
            break