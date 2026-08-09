export const resolveRendererRoutes = (routesOrFactory, windowType = 'editor') => {
  const routeRecords = typeof routesOrFactory === 'function'
    ? routesOrFactory(windowType)
    : routesOrFactory

  if (!Array.isArray(routeRecords)) {
    throw new TypeError('Renderer router expected an array of route records.')
  }

  return routeRecords
}

export default resolveRendererRoutes
