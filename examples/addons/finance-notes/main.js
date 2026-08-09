self.elephantAddon = {
  activate(api) {
    return api.commands.register({
      id: 'com.elephantnote.examples.finance-notes.refresh',
      title: 'Refresh finance note',
      description: 'Fetch a current Yahoo Finance chart snapshot and write a Markdown note.',
      async run(input = {}) {
        const symbol = String(input.symbol || 'AAPL').toUpperCase().replace(/[^A-Z0-9.^-]/g, '')
        if (!symbol) throw new Error('A valid market symbol is required')

        const response = await api.http.request({
          url: `https://query1.finance.yahoo.com/v8/finance/chart/${encodeURIComponent(symbol)}?range=5d&interval=1d`,
          method: 'GET',
          headers: {
            Accept: 'application/json'
          }
        })

        if (!response.ok) {
          throw new Error(`Finance provider returned HTTP ${response.status}`)
        }

        const payload = JSON.parse(response.body)
        const result = payload?.chart?.result?.[0]
        const meta = result?.meta || {}
        const closes = result?.indicators?.quote?.[0]?.close || []
        const latestClose = [...closes].reverse().find((value) => Number.isFinite(value))
        const currency = meta.currency || ''
        const fetchedAt = new Date().toISOString()
        const path = `Finance/${symbol}.md`

        const markdown = [
          `# ${symbol}`,
          '',
          `- **Latest close:** ${Number.isFinite(latestClose) ? latestClose : 'Unavailable'} ${currency}`,
          `- **Exchange:** ${meta.exchangeName || 'Unknown'}`,
          `- **Instrument:** ${meta.instrumentType || 'Unknown'}`,
          `- **Fetched:** ${fetchedAt}`,
          '',
          '## Source',
          '',
          `- Yahoo Finance chart API for \`${symbol}\``,
          '- This note is generated data, not financial advice.',
          ''
        ].join('\n')

        await api.notes.write(path, markdown)
        await api.storage.set('lastRun', { symbol, path, fetchedAt, latestClose })
        return { symbol, path, fetchedAt, latestClose }
      }
    })
  }
}
