# Replaces devspace.yaml. default_registry() gives a real blanket registry
# rewrite (unlike DevSpace's exact-string-match substitution), which is what
# made the REGISTRY-override workaround in ../REGISTRY-OVERRIDE-NOTES.md
# unnecessary here.
#
# REGISTRY must be a canonical host if overridden (e.g. docker.io/tawalaya, not
# bare tawalaya) — Tilt validates this and normalizes docker.io refs back down
# to the short form on its own.

DEFAULT_REGISTRY = 'ghcr.io/ise-tu-berlin/sustainable-otel'
REGISTRY = os.getenv('REGISTRY', '')
OVERLAY = os.getenv('OVERLAY', 'local')

# ponytail: only rewrite when explicitly overridden — default_registry() munges
# '/' -> '_' in the image name, so leaving it off for the default registry keeps
# the deployed image name identical to what's hardcoded in k8s/base today.
if REGISTRY and REGISTRY != DEFAULT_REGISTRY:
    default_registry(REGISTRY)

# Tilt only extracts image refs from kinds it already knows (Deployment,
# StatefulSet, ...). Without this it cannot find the image nested in a Knative
# Service's spec.template.spec.containers, reports otel-quote as unused, and
# silently never builds it. Must come before k8s_yaml().
k8s_kind('Service', api_version='serving.knative.dev/v1',
         image_json_path='{.spec.template.spec.containers[0].image}')

k8s_yaml(kustomize('k8s/overlays/' + OVERLAY))

# DevSpace ran this as a global before:deploy hook, which blocked every
# resource. As a resource_dep only quoteservice waits on it.
local_resource(
    'install-knative-local',
    cmd='bash k8s/scripts/install-knative-local.sh',
)

SERVICES = [
    'cart',
    'checkout',
    'currency',
    'email',
    'flagd-ui',
    'frontend',
    'frontend-proxy',
    'image-provider',
    'load-generator',
    'payment',
    'product-catalog',
    'quote',
    'recommendation',
    'shipping',
]

# cart's Dockerfile sits one level deeper than every other service's.
DOCKERFILES = {'cart': 'src/cart/src/Dockerfile'}

for name in SERVICES:
    docker_build(
        DEFAULT_REGISTRY + '/otel-' + name,
        context='.',
        dockerfile=DOCKERFILES.get(name, 'src/' + name + '/Dockerfile'),
        # Every Dockerfile needs the repo root as context but only COPYs
        # ./src/<name>/ and ./pb/. Without only=, a single edit anywhere
        # invalidates every image on every watch tick.
        only=['src/' + name, 'pb'],
    )

# The one port DevSpace forwarded (dev.frontendProxy).
k8s_resource('frontend-proxy', port_forwards=8080)

# quote is the serverless tactic: a Knative Service that scales to zero, so it
# needs Knative Serving's CRDs and controller to exist before it is applied.
k8s_resource('quoteservice', resource_deps=['install-knative-local'])
