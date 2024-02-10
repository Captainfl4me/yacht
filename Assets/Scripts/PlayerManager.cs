using System;
using Unity.Netcode;
using UnityEngine;

public class PlayerManager : NetworkBehaviour
{
    // Network Variables
    private NetworkVariable<Vector3> _networkPosition = new();
    private NetworkVariable<float> _networkRotation = new(writePerm: NetworkVariableWritePermission.Owner);

    private AudioListener audioListener;
    private Camera playerCamera;

    public float mouseSensitivity = 4f;

    [Header("UI Elements")]
    [SerializeField] Sprite grabableSprite;
    [SerializeField] Sprite grabSprite;
    [SerializeField] GameObject aimPrefab;
    [SerializeField] GameObject iconPrefab;

    [Header("Grabbing settings")]
    [SerializeField] Transform grabPoint;
    [SerializeField] float maxGrabDistance = 10f;
    [SerializeField] float grabForce = 150f;
    [SerializeField] float grabDamping = 1f;
    private bool lastGrabErrorInitialized = false;
    private Vector3 lastGrabError = Vector3.zero;

    private GameObject grabbedObject = null;
    private Rigidbody grabbedRigidbody = null;
    private Vector3 grabbedPointRelative;

    private void Awake()
    {
        Debug.Log("PlayerManager Awake");
        audioListener = GetComponentInChildren<AudioListener>();
        playerCamera = GetComponentInChildren<Camera>();

        Cursor.lockState = CursorLockMode.Locked;
    }

    private void Update()
    {
        if (IsServer)
        {
            _networkPosition.Value = transform.position;
        }
        else
        {
            transform.position = _networkPosition.Value;
        }
        if (IsOwner)
        {
            float yRotation = transform.eulerAngles.y + Input.GetAxis("Mouse X") * mouseSensitivity;
            transform.localRotation = Quaternion.Euler(transform.eulerAngles.x, yRotation, 0f);

            float xRotation = playerCamera.transform.localEulerAngles.x - Input.GetAxis("Mouse Y") * mouseSensitivity;
            playerCamera.transform.localRotation = Quaternion.Euler(xRotation, playerCamera.transform.localEulerAngles.y, 0f);
            _networkRotation.Value = transform.rotation.eulerAngles.y;
        }
        else
        {
            transform.eulerAngles = new Vector3(transform.eulerAngles.x, _networkRotation.Value, transform.eulerAngles.z);
        }
    }

    private void FixedUpdate()
    {
        if (!IsOwner) return;

        if (grabbedObject == null)
        {
            Ray ray = playerCamera.ScreenPointToRay(new Vector3(playerCamera.scaledPixelWidth / 2, playerCamera.scaledPixelHeight / 2, 0));
            int layerMask = 1 << 6; // Layer 6 is the grabable layer
            if (Physics.Raycast(ray, out RaycastHit hit, maxGrabDistance, layerMask))
            {
                Debug.DrawRay(ray.GetPoint(0), ray.direction * hit.distance, Color.green);
                aimPrefab.SetActive(false);
                iconPrefab.SetActive(true);

                if (Input.GetMouseButton(0))
                {
                    if (GrabItem(hit.collider.gameObject))
                    {
                        Debug.Log("Grabbing " + grabbedObject.name);
                        iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = grabSprite;
                        grabbedPointRelative = grabbedObject.transform.InverseTransformPoint(ray.GetPoint(hit.distance));
                        lastGrabError = Vector3.zero;
                        lastGrabErrorInitialized = false;
                    }
                }
            }
            else
            {
                Debug.DrawRay(ray.GetPoint(0), ray.direction * maxGrabDistance, Color.white);
                aimPrefab.SetActive(true);
                iconPrefab.SetActive(false);
            }
        }
        else
        {
            // PD controller
            Vector3 error = grabPoint.position - grabbedObject.transform.TransformPoint(grabbedPointRelative);
            Vector3 PForce = grabForce * error;
            Vector3 DForce = lastGrabErrorInitialized ? grabDamping * (error - lastGrabError) / Time.fixedDeltaTime : Vector3.zero;
            lastGrabError = error;
            lastGrabErrorInitialized = true;

            grabbedRigidbody.AddForce(PForce + DForce);

            if (!Input.GetMouseButton(0))
            {
                if (DropItem()) 
                {
                    Debug.Log("Dropping item");
                    iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = grabableSprite;
                }
            }
        }
    }

    private bool GrabItem(GameObject item)
    {
        if (grabbedObject != null) return false;

        grabbedRigidbody = item.GetComponent<Rigidbody>();
        if (grabbedRigidbody == null) 
        {
            grabbedRigidbody = item.GetComponentInParent<Rigidbody>();
            if (grabbedRigidbody == null) return false;
        }

        grabbedObject = grabbedRigidbody.gameObject;
        grabbedRigidbody.useGravity = false;
        grabbedRigidbody.constraints = RigidbodyConstraints.FreezeRotation;
        // grabbedObject.transform.SetParent(grabPoint);
        return true;
    }
    private bool DropItem()
    {
        if (grabbedObject == null) return false;

        grabbedRigidbody.useGravity = true;
        grabbedRigidbody.constraints = RigidbodyConstraints.None;
        // grabbedObject.transform.SetParent(null);
        grabbedRigidbody = null;
        grabbedObject = null;
        return true;
    }

    public override void OnNetworkSpawn()
    {
        if (!IsOwner)
        {
            audioListener.enabled = false;
            playerCamera.enabled = false;
        }
    }
}
